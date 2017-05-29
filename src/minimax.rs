//! An implementation of the minimax algorithm.

use state::State;
use state::CheckBox;
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
        self.current_state.max_depth(0)
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
        }).expect("subsequent_states failed to calculate a possible move?");

        self.current_state = new_state.take();
        Ok(())
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

    fn max_depth(&self, this_depth: usize) -> usize {
        use std::cmp;

        let mut max = this_depth;

        if let Some(ref children) = self.children {
            for child in children.iter() {
                max = cmp::max(max, child.max_depth(this_depth + 1));
            }
        }

        max
    }

    /// Ensures to have computed the children states for this state.
    fn ensure_children(&mut self) -> &mut [MiniMaxNode] {
        if self.children.is_none() {
            let children =
                if self.state.score() != 0 {
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
