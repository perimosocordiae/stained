use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    players: Vec<Player>,
    curr_player_idx: usize,
    dice_bag: Vec<Color>,
    // TODO: tools: Vec<Tool>,
    // TODO: objectives: Vec<Objective>,
    round: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    tokens: u8,
    board: [[BoardCell; 5]; 5],
    secret: Color,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BoardCell {
    Dice(Dice),
    Slot(Slot),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dice {
    color: Color,
    face: u8,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Slot {
    Any,
    Color(Color),
    Face(u8),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
}

pub fn roll_die(color: Color) -> Dice {
    Dice {
        color,
        face: rand::random::<u8>() % 6 + 1,
    }
}
