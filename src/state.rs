/// The state of a given box in the tic-tac-toe game.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(i8)]
enum CheckBox {
    Empty = 0,
    X = -10,
    O = 10,
}

#[derive(Clone, Debug)]
struct State {
    field: [[CheckBox; 3]; 3],
}

impl State {
    fn new() -> Self {
        Self {
            field: [
                [CheckBox::Empty, CheckBox::Empty, CheckBox::Empty],
                [CheckBox::Empty, CheckBox::Empty, CheckBox::Empty],
                [CheckBox::Empty, CheckBox::Empty, CheckBox::Empty],
            ],
        }
    }

    /// For a given state, iterate over all the possible child states created by
    /// a single move of the piece `c`, which can't be empty.
    fn subsequent_states<'a>(
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
    fn score(&self) -> i8 {
        return self.row_score(0) +
            self.row_score(1) +
            self.row_score(2) +
            self.column_score(0) +
            self.column_score(1) +
            self.column_score(2) +
            self.main_diagonal_score() +
            self.cross_diagonal_score();
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
            if self.field[i][3 - i] != center {
                return 0;
            }
        }
        return center as i8
    }
}

struct SubsequentStatesIterator<'a> {
    initial_state: &'a State,
    row: usize,
    col: usize,
    player: CheckBox,
}

impl<'a> Iterator for SubsequentStatesIterator<'a> {
    type Item = State;

    fn next(&mut self) -> Option<Self::Item> {
        while self.row != 3 || self.col != 3 {
            if self.col == 3 {
                self.row += 1;
                self.col = 0;
            }
            if self.initial_state.field[self.row][self.col] == CheckBox::Empty {
                self.col += 1;
                let mut ret = self.initial_state.clone();
                ret.field[self.row][self.col] = self.player;
                return Some(ret)
            }
            self.col += 1;
        }

        None
    }
}
