use crate::board::BoardCell;
use crate::color::{Color, Dice, ALL_COLORS};
use crate::constants::{BOARD_COLS, BOARD_ROWS};
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
impl Objective {
    pub fn score(self, board: &[[BoardCell; BOARD_COLS]; BOARD_ROWS]) -> i32 {
        match self {
            Objective::ColumnNumbers(_n) => {
                todo!("ColumnNumbers")
            }
            Objective::RowNumbers(_n) => {
                todo!("RowNumbers")
            }
            Objective::Numbers(n) => n * (1..=6).map(|i| count_number(board, i)).min().unwrap_or(0),
            Objective::ColumnColors(_n) => {
                todo!("ColumnColors")
            }
            Objective::RowColors(_n) => {
                todo!("RowColors")
            }
            Objective::Colors(n) => {
                n * ALL_COLORS
                    .iter()
                    .map(|&c| count_color(board, c))
                    .min()
                    .unwrap_or(0)
            }
            Objective::Pair12(n) => n * count_number(board, 1).min(count_number(board, 2)),
            Objective::Pair34(n) => n * count_number(board, 3).min(count_number(board, 4)),
            Objective::Pair56(n) => n * count_number(board, 5).min(count_number(board, 6)),
            Objective::ColorDiagonals(_n) => {
                todo!("ColorDiagonals")
            }
        }
    }
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
        .filter(|cell| matches!(cell.die, Some(Dice { color, .. }) if color == c))
        .count() as i32
}
