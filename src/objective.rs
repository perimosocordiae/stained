use crate::board::BoardCell;
use crate::color::{ALL_COLORS, Color, Dice};
use crate::constants::{BOARD_COLS, BOARD_ROWS, NUM_COLORS};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Objective {
    ColumnNumbers(i32),
    RowNumbers(i32),
    Numbers(i32),
    ColumnColors(i32),
    RowColors(i32),
    Colors(i32),
    Pair12(i32),
    Pair34(i32),
    Pair56(i32),
    ColorDiagonals(i32),
}

pub const ALL_OBJECTIVES: [Objective; 10] = [
    Objective::ColumnNumbers(4),
    Objective::RowNumbers(5),
    Objective::Numbers(5),
    Objective::ColumnColors(5),
    Objective::RowColors(6),
    Objective::Colors(4),
    Objective::Pair12(2),
    Objective::Pair34(2),
    Objective::Pair56(2),
    Objective::ColorDiagonals(1),
];

impl Objective {
    pub fn score(self, board: &[[BoardCell; BOARD_COLS]; BOARD_ROWS]) -> i32 {
        match self {
            Objective::ColumnNumbers(n) => {
                n * (0..BOARD_COLS)
                    .filter(|c| {
                        distinct_numbers(board.iter().map(|row| &row[*c]))
                    })
                    .count() as i32
            }
            Objective::RowNumbers(n) => {
                n * board
                    .iter()
                    .filter(|row| distinct_numbers(row.iter()))
                    .count() as i32
            }
            Objective::Numbers(n) => {
                n * (1..=6).map(|i| count_number(board, i)).min().unwrap_or(0)
            }
            Objective::ColumnColors(n) => {
                n * (0..BOARD_COLS)
                    .filter(|c| {
                        distinct_colors(board.iter().map(|row| &row[*c]))
                    })
                    .count() as i32
            }
            Objective::RowColors(n) => {
                n * board
                    .iter()
                    .filter(|row| distinct_colors(row.iter()))
                    .count() as i32
            }
            Objective::Colors(n) => {
                n * ALL_COLORS
                    .iter()
                    .map(|&c| count_color(board, c))
                    .min()
                    .unwrap_or(0)
            }
            Objective::Pair12(n) => {
                n * count_number(board, 1).min(count_number(board, 2))
            }
            Objective::Pair34(n) => {
                n * count_number(board, 3).min(count_number(board, 4))
            }
            Objective::Pair56(n) => {
                n * count_number(board, 5).min(count_number(board, 6))
            }
            Objective::ColorDiagonals(n) => n * color_diagonals(board),
        }
    }
}

fn distinct_numbers<'a>(cells: impl Iterator<Item = &'a BoardCell>) -> bool {
    let mut seen = [false; 6];
    for cell in cells {
        if let Some(Dice { face, .. }) = cell.die {
            let idx = face as usize - 1;
            if seen[idx] {
                return false;
            }
            seen[idx] = true;
        } else {
            return false;
        }
    }
    true
}
fn distinct_colors<'a>(cells: impl Iterator<Item = &'a BoardCell>) -> bool {
    let mut seen = [false; NUM_COLORS];
    for cell in cells {
        if let Some(Dice { color, .. }) = cell.die {
            let idx = color as usize;
            if seen[idx] {
                return false;
            }
            seen[idx] = true;
        } else {
            return false;
        }
    }
    true
}

fn count_number(board: &[[BoardCell; BOARD_COLS]; BOARD_ROWS], n: u8) -> i32 {
    board
        .iter()
        .flatten()
        .filter(|cell| matches!(cell.die, Some(Dice { face, .. }) if face == n))
        .count() as i32
}
fn count_color(board: &[[BoardCell; BOARD_COLS]; BOARD_ROWS], c: Color) -> i32 {
    board
        .iter()
        .flatten()
        .filter(
            |cell| matches!(cell.die, Some(Dice { color, .. }) if color == c),
        )
        .count() as i32
}

fn color_diagonals(board: &[[BoardCell; BOARD_COLS]; BOARD_ROWS]) -> i32 {
    let mut count = 0;
    for i in 0..BOARD_ROWS {
        for j in 0..BOARD_COLS {
            if let Some(d) = board[i][j].die
                && (
                    // Check upper-left and upper-right
                    (i > 0 && has_diag(&board[i - 1], j, d.color)) ||
                    // Check lower-left and lower-right
                    (i < BOARD_ROWS - 1 && has_diag(&board[i + 1], j, d.color))
                )
            {
                count += 1;
            }
        }
    }
    count
}

fn has_diag(row: &[BoardCell], j: usize, color: Color) -> bool {
    (j > 0 && matches!(row[j - 1].die, Some(d) if d.color == color))
        || (j < BOARD_COLS - 1
            && matches!(row[j + 1].die, Some(d) if d.color == color))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_diag() {
        let empty = BoardCell::default();
        let red = BoardCell::with_die(Dice {
            color: Color::Red,
            face: 3,
        });
        let blue = BoardCell::with_die(Dice {
            color: Color::Blue,
            face: 4,
        });
        let row = [empty.clone(), red, blue, empty.clone(), empty];

        assert!(has_diag(&row, 0, Color::Red), "red @ 0 has diag");
        assert!(!has_diag(&row, 0, Color::Blue), "blue @ 0 no diag");
        assert!(!has_diag(&row, 1, Color::Red), "red @ 1 no diag");
        assert!(has_diag(&row, 1, Color::Blue), "blue @ 1 has diag");
        assert!(has_diag(&row, 2, Color::Red), "red @ 2 has diag");
        assert!(!has_diag(&row, 2, Color::Blue), "blue @ 2 no diag");
        assert!(!has_diag(&row, 3, Color::Red), "red @ 3 no diag");
        assert!(has_diag(&row, 3, Color::Blue), "blue @ 3 has diag");
    }

    #[test]
    fn test_color_diagonals() {
        let empty = BoardCell::default();
        let red = BoardCell::with_die(Dice {
            color: Color::Red,
            face: 3,
        });
        let blue = BoardCell::with_die(Dice {
            color: Color::Blue,
            face: 4,
        });
        // Board layout:
        // . R . . .
        // R . B . .
        // . B . R .
        // . . R . B
        let board = [
            [
                empty.clone(),
                red.clone(),
                empty.clone(),
                empty.clone(),
                empty.clone(),
            ],
            [
                red.clone(),
                empty.clone(),
                blue.clone(),
                empty.clone(),
                empty.clone(),
            ],
            [
                empty.clone(),
                blue.clone(),
                empty.clone(),
                red.clone(),
                empty.clone(),
            ],
            [
                empty.clone(),
                empty.clone(),
                red.clone(),
                empty.clone(),
                blue.clone(),
            ],
        ];

        assert_eq!(color_diagonals(&board), 6);
        assert_eq!(Objective::ColorDiagonals(2).score(&board), 12);
    }
}
