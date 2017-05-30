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

use std::fmt;

/// The state of a given box in the tic-tac-toe game.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(i8)]
pub enum CheckBox {
    Empty = 0,
    X = -10,
    O = 10,
}

impl CheckBox {
    /// Returns the next player, if `self` has moved.
    pub fn next_player(&self) -> Self {
        match *self {
            CheckBox::Empty => CheckBox::Empty,
            CheckBox::X => CheckBox::O,
            CheckBox::O => CheckBox::X,
        }
    }

    fn dump_char(&self) -> char {
        match *self {
            CheckBox::Empty => '_',
            CheckBox::X => 'X',
            CheckBox::O => 'O',
        }
    }
}

#[derive(Clone, Debug)]
pub struct State {
    field: [[CheckBox; 3]; 3],
}

impl State {
    pub fn initial() -> Self {
        Self {
            field: [
                [CheckBox::Empty, CheckBox::Empty, CheckBox::Empty],
                [CheckBox::Empty, CheckBox::Empty, CheckBox::Empty],
                [CheckBox::Empty, CheckBox::Empty, CheckBox::Empty],
            ],
        }
    }

    pub fn dump<W>(&self, indent: usize, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        for i in 0..3 {
            for _ in 0..indent {
                dest.write_char(' ')?;
            }
            self.dump_row(i, dest)?;
            dest.write_char('\n')?;
        }

        Ok(())
    }

    pub fn dump_row<W>(&self, index: usize, dest: &mut W) -> fmt::Result
        where W: fmt::Write,
    {
        let row = self.field[index];
        dest.write_char('[')?;
        dest.write_char(row[0].dump_char())?;
        dest.write_char(' ')?;
        dest.write_char(row[1].dump_char())?;
        dest.write_char(' ')?;;
        dest.write_char(row[2].dump_char())?;
        dest.write_char(']')
    }


    /// For a given state, iterate over all the possible child states created by
    /// a single move of the piece `c`, which can't be empty.
    pub fn subsequent_states<'a>(
        &'a self,
        player: CheckBox
    ) -> SubsequentStatesIterator<'a> {
        assert!(player != CheckBox::Empty);

        SubsequentStatesIterator {
            initial_state: self,
            row: 0,
            col: 0,
            player: player,
        }
    }

    /// TODO(emilio): This can be much more efficient, but you know...
    pub fn score(&self) -> i8 {
        macro_rules! return_if_nonzero {
            ($e:expr) => {
                {
                    let v = $e;
                    if v != 0 {
                        return v;
                    }
                }
            }
        }
        return_if_nonzero!(self.row_score(0));
        return_if_nonzero!(self.row_score(1));
        return_if_nonzero!(self.row_score(2));
        return_if_nonzero!(self.column_score(0));
        return_if_nonzero!(self.column_score(1));
        return_if_nonzero!(self.column_score(2));
        return_if_nonzero!(self.main_diagonal_score());
        return_if_nonzero!(self.cross_diagonal_score());
        0
    }

    fn row_score(&self, row: usize) -> i8 {
        let row = self.field[row];
        let first = row[0];
        if row.iter().all(|i| *i == first) {
            return first as i8
        }
        0
    }

    fn column_score(&self, col: usize) -> i8 {
        let first = self.field[0][col];
        for i in 1..3 {
            if self.field[i][col] != first {
                return 0;
            }
        }

        return first as i8
    }

    pub fn get(&self, x: usize, y: usize) -> CheckBox {
        self.field[x][y]
    }

    fn main_diagonal_score(&self) -> i8 {
        let center = self.field[1][1];
        for i in 0..3 {
            if self.field[i][i] != center {
                return 0;
            }
        }

        return center as i8
    }

    fn cross_diagonal_score(&self) -> i8 {
        let center = self.field[1][1];
        for i in 0..3 {
            if self.field[i][3 - i - 1] != center {
                return 0;
            }
        }
        return center as i8
    }
}

pub struct SubsequentStatesIterator<'a> {
    initial_state: &'a State,
    row: usize,
    col: usize,
    player: CheckBox,
}

impl<'a> Iterator for SubsequentStatesIterator<'a> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        while self.row != 3 {
            if self.col == 3 {
                self.row += 1;
                self.col = 0;
                continue;
            }
            if self.initial_state.field[self.row][self.col] == CheckBox::Empty {
                self.col += 1;
                let mut ret = self.initial_state.clone();
                ret.field[self.row][self.col - 1] = self.player;
                return Some(ret)
            }
            self.col += 1;
        }

        None
    }
}
