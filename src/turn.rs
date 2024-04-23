use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TurnPhase {
    SelectTemplate,
    FirstDraft,
    SecondDraft,
    GameOver,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnAction {
    pub idx: ActionType,
    pub coords: Option<(usize, usize)>,
}
impl TurnAction {
    pub fn pass() -> Self {
        Self {
            idx: ActionType::DraftDie(0),
            coords: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    SelectTemplate(usize),
    DraftDie(usize),
    UseTool(usize),
}
