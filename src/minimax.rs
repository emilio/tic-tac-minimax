//! An implementation of the minimax algorithm.

use state::State;
use state::CheckBox;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug)]
pub struct MiniMaxTree {
    current_state: MiniMaxNode,
}

impl MiniMaxTree {
    pub fn new(player: CheckBox) -> Self {
        Self {
            current_state: MiniMaxNode {
                state: State::initial(),
                player: player,
                children: None,
            },
        }
    }

    /// Ensures we have computed at least up to `depth` levels in the tree.
    pub fn update_to_depth(&mut self, depth: usize) {
        self.current_state.update(0, depth);
    }

    /// Ensures we have computed at least up to `depth` levels in the tree.
    pub fn dump<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        self.current_state.dump(0, dest)
    }

    /// Ensures we have computed at least up to `depth` levels in the tree.
    pub fn depth(&self) -> usize {
        self.current_state.max_computed_depth(0)
    }

    /// Returns the current state of the game.
    pub fn state(&self) -> &State {
        &self.current_state.state
    }

    /// Toggles the square at (x, y).
    ///
    /// Returns an error if the square was not empty.
    pub fn choose(&mut self, x: usize, y: usize) -> Result<(), ()> {
        if self.current_state.state.get(x, y) != CheckBox::Empty {
            return Err(());
        }

        let current_player = self.current_state.player;
        let mut current_state = self.current_state.take();
        let mut new_state = current_state.ensure_children().iter_mut().find(|s| {
            s.state.get(x, y) == current_player
        });

        let new_state = match new_state {
            None => {
                // If we couldn't find a state here, it means that the game is
                // over, compute it from the subsequents states.
                //
                // NOTE(emilio): We don't really need to iterate this, but seems
                // cheap enough.
                let s = self.current_state.state.subsequent_states(current_player).find(|s| {
                    s.get(x, y) == current_player
                }).unwrap();
                assert_ne!(s.score(), 0);
                MiniMaxNode::new(s, current_player.next_player())
            }
            Some(mut new_state) => new_state.take(),
        };

        self.current_state = new_state;

        Ok(())
    }

    pub fn choose_with_index(&mut self, index: usize) {
        let mut current_state = self.current_state.take();
        let mut new_state = &mut current_state.ensure_children()[index];
        self.current_state = new_state.take();
    }

    /// Finds a min/max move index for the next round.
    ///
    /// Returns `None` if the game is already over, or if `max_depth` is zero.
    pub fn find_move_index(
        &mut self,
        max_depth: usize)
        -> Option<usize>
    {
        use std::i8;

        if self.current_state.score() != 0 || max_depth == 0 {
            // It's over already, or we didn't have any chances of computing it.
            return None;
        }

        let maximizing = self.current_state.player as i8 > 0;

        let mut best = if maximizing { i8::MIN } else { i8::MAX };
        let mut best_move = None;

        for (i, child) in self.current_state.ensure_children().iter_mut().enumerate() {
            let child_score = child.minimax(max_depth - 1);
            let child_is_best_so_far = if maximizing {
                child_score > best
            } else {
                child_score < best
            };

            if child_is_best_so_far {
                best = child_score;
                best_move = Some(i);
            }
        }

        best_move
    }
}

#[derive(Debug)]
struct MiniMaxNode {
    /// The state this node represents.
    state: State,
    /// The player that has to move.
    player: CheckBox,
    /// The children of the node. This will be `None` when they haven't been
    /// computed yet.
    children: Option<Box<[MiniMaxNode]>>,
}

impl MiniMaxNode {
    pub fn new(state: State, player: CheckBox) -> Self {
        Self {
            state: state,
            player: player,
            children: None,
        }
    }

    /// Gets a copy of this state, "taking" the computing subtree.
    fn take(&mut self) -> Self {
        Self {
            state: self.state.clone(),
            player: self.player,
            children: self.children.take(),
        }
    }

    /// Ensures we've computed children up to depth `max_depth`.
    fn update(&mut self, depth: usize, max_depth: usize) {
        if depth == max_depth {
            return;
        }

        for child in self.ensure_children() {
            child.update(depth + 1, max_depth);
        }
    }

    fn max_computed_depth(&self, this_depth: usize) -> usize {
        use std::cmp;

        let mut max = this_depth;

        if let Some(ref children) = self.children {
            for child in children.iter() {
                max = cmp::max(max, child.max_computed_depth(this_depth + 1));
            }
        }

        max
    }

    fn minimax(&mut self, max_depth: usize) -> i8 {
        use std::{cmp, i8};

        if max_depth == 0 {
            return self.score();
        }

        if self.ensure_children().is_empty() {
            return self.state.score();
        }

        let maximizing = self.player as i8 > 0;
        let children = self.ensure_children();

        let mut best = if maximizing { i8::MIN } else { i8::MAX };
        for child in children {
            let val = child.minimax(max_depth - 1);
            best = if maximizing {
                cmp::max(val, best)
            } else {
                cmp::min(val, best)
            };
        }
        best
    }

    /// Ensures to have computed the children states for this state.
    fn ensure_children(&mut self) -> &mut [MiniMaxNode] {
        if self.children.is_none() {
            let children =
                if self.score() != 0 {
                    // This is a game over state, so just prune here.
                    vec![].into_boxed_slice()
                } else {
                    self.state.subsequent_states(self.player).map(|s| {
                        MiniMaxNode::new(s, self.player.next_player())
                    }).collect::<Vec<_>>().into_boxed_slice()
                };

            self.children = Some(children);
        }
        self.children.as_mut().unwrap()
    }

    fn score(&self) -> i8 {
        self.state.score()
    }

    pub fn dump<W>(&self, indent: usize, dest: &mut W) -> fmt::Result
        where W: fmt::Write
    {
        self.state.dump(indent, dest)?;
        for _ in 0..indent {
            dest.write_char(' ')?;
        }

        writeln!(dest, "score: {}", self.score())?;

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.dump(indent + 1, dest)?;
            }
        }

        Ok(())
    }
}
