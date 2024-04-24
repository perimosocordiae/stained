use crate::constants::NUM_COLORS;
use rand::prelude::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Color {
    Red,
    Yellow,
    Green,
    Blue,
    Purple,
}
impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let color = match self {
            Color::Red => "R",
            Color::Yellow => "Y",
            Color::Green => "G",
            Color::Blue => "B",
            Color::Purple => "P",
        };
        write!(f, "{}", color)
    }
}
pub const ALL_COLORS: [Color; NUM_COLORS] = [
    Color::Red,
    Color::Yellow,
    Color::Green,
    Color::Blue,
    Color::Purple,
];

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dice {
    pub color: Color,
    pub face: u8,
}
impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.color, self.face)
    }
}
impl Dice {
    pub fn roll(color: Color, rng: &mut impl rand::Rng) -> Self {
        let face = (1..=6).choose(rng).unwrap_or(1);
        Self { color, face }
    }
    pub fn reroll(&mut self, rng: &mut impl rand::Rng) {
        self.face = (1..=6).choose(rng).unwrap_or(1);
    }
    pub fn flip(&mut self) {
        self.face = 7 - self.face;
    }
}
