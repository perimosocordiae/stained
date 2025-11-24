use crate::color::Dice;
use crate::template::Slot;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoardCell {
    pub slot: Slot,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub die: Option<Dice>,
}
impl Default for BoardCell {
    fn default() -> Self {
        Self {
            slot: Slot::Any,
            die: None,
        }
    }
}
impl Display for BoardCell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(die) = self.die {
            write!(f, "{}", die)
        } else {
            write!(f, "{}", self.slot)
        }
    }
}
impl BoardCell {
    pub fn with_die(die: Dice) -> Self {
        Self {
            slot: Slot::Any,
            die: Some(die),
        }
    }
}
