use crate::constants::NUM_COLORS;
use rand::prelude::IteratorRandom;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Color {
    #[serde(rename = "R")]
    Red,
    #[serde(rename = "Y")]
    Yellow,
    #[serde(rename = "G")]
    Green,
    #[serde(rename = "B")]
    Blue,
    #[serde(rename = "P")]
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
impl FromStr for Color {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "R" => Ok(Color::Red),
            "Y" => Ok(Color::Yellow),
            "G" => Ok(Color::Green),
            "B" => Ok(Color::Blue),
            "P" => Ok(Color::Purple),
            _ => Err(format!("Invalid color string: {}", s)),
        }
    }
}
pub const ALL_COLORS: [Color; NUM_COLORS] = [
    Color::Red,
    Color::Yellow,
    Color::Green,
    Color::Blue,
    Color::Purple,
];

#[derive(Debug, Clone, Copy)]
pub struct Dice {
    pub color: Color,
    pub face: u8,
}
impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.color, self.face)
    }
}
impl Serialize for Dice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = format!("{}{}", self.color, self.face);
        serializer.serialize_str(&s)
    }
}
impl<'de> Deserialize<'de> for Dice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        if s.len() != 2 {
            return Err(serde::de::Error::custom(format!(
                "Invalid dice format: {s}"
            )));
        }
        let color = s[0..1].parse::<Color>().map_err(|_| {
            serde::de::Error::custom(format!("Invalid color: {s}"))
        })?;
        let face = s[1..2].parse::<u8>().map_err(|_| {
            serde::de::Error::custom(format!("Invalid face: {s}"))
        })?;
        Ok(Dice { color, face })
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
    pub fn increment(&mut self) {
        self.face = self.face.saturating_add(1);
    }
    pub fn decrement(&mut self) {
        self.face = self.face.saturating_sub(1);
    }
}

#[test]
fn test_dice_serialization() {
    let die = Dice {
        color: Color::Blue,
        face: 5,
    };
    let serialized = serde_json::to_string(&die).unwrap();
    assert_eq!(serialized, r#""B5""#);
    let deserialized: Dice = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.color, Color::Blue);
    assert_eq!(deserialized.face, 5);
}
