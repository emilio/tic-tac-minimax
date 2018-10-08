/*
 * Copyright (C) 2017 Emilio Cobos Álvarez <emilio@crisal.io>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

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
    #[allow(dead_code)] // This is just for debugging.
    pub fn dump<W>(&self, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        self.current_state.dump(0, dest)
    }

    /// Returns the current state of the game.
    pub fn state(&self) -> &State {
        &self.current_state.state
    }

    /// Toggles the square at (x, y).
    ///
    /// Returns an error if the square was not empty.
    pub fn choose(&mut self, x: usize, y: usize) -> Result<(), ()> {
        if self.current_state.state.get(x, y) != CheckBox::Empty ||
            self.current_state.score() != 0 {
            return Err(());
        }

        let current_player = self.current_state.player;
        let mut current_state = self.current_state.take();
        let new_state = current_state.ensure_children().iter_mut().find(|s| {
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
        let new_state = &mut current_state.ensure_children()[index];
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
        let mut nodes_visited_pruning = 0;
        let move_pruning = self.find_move_index_internal(
            max_depth,
            /* prune = */ true,
            &mut nodes_visited_pruning);

        if cfg!(debug_assertions) {
            let mut nodes_visited_without_pruning = 0;
            let move_without_pruning = self.find_move_index_internal(
                max_depth,
                /* prune = */ false,
                &mut nodes_visited_without_pruning,
            );

            // This is the whole point of it!
            assert_eq!(move_pruning, move_without_pruning);
            assert!(nodes_visited_pruning <= nodes_visited_without_pruning);
        }

        move_pruning
    }

    fn find_move_index_internal(
        &mut self,
        max_depth: usize,
        prune: bool,
        nodes_visited: &mut usize,
    ) -> Option<usize>
    {
        use std::i8;
        *nodes_visited = 0;

        if self.current_state.score() != 0 || max_depth == 0 {
            // It's over already, or we didn't have any chances of computing it.
            return None;
        }

        let maximizing = self.current_state.player as i8 > 0;

        let mut best = if maximizing { i8::MIN } else { i8::MAX };
        let mut best_move = None;

        let mut alpha = i8::MIN;
        let mut beta = i8::MAX;

        for (i, child) in self.current_state.ensure_children().iter_mut().enumerate() {
            let child_score = child.minimax(
                max_depth - 1,
                alpha,
                beta,
                prune,
                nodes_visited
            );

            let child_is_best_so_far = if maximizing {
                child_score > best
            } else {
                child_score < best
            };

            if child_is_best_so_far {
                best = child_score;
                if maximizing {
                    alpha = best;
                } else {
                    beta = best;
                }
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

    fn minimax(
        &mut self,
        max_depth: usize,
        mut alpha: i8,
        mut beta: i8,
        prune: bool,
        nodes_visited: &mut usize,
    ) -> i8 {
        use std::{cmp, i8};
        *nodes_visited += 1;

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
            let val = child.minimax(
                max_depth - 1,
                alpha,
                beta,
                prune,
                nodes_visited
            );

            best = if maximizing {
                cmp::max(val, best)
            } else {
                cmp::min(val, best)
            };

            if maximizing {
                if best > beta && prune {
                    return best;
                }
                alpha = cmp::max(best, alpha);
            } else {
                if best < alpha && prune {
                    return best;
                }
                beta = cmp::min(best, beta);
            }
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
