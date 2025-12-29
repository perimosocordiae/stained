use serde::{Deserialize, Serialize};

use crate::tool::{Tool, ToolData, ToolType};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
    pub tool: Option<ToolData>,
}
impl TurnAction {
    pub fn pass() -> Self {
        Self {
            idx: ActionType::DraftDie(0, None),
            coords: None,
            tool: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionType {
    SelectTemplate(usize),
    // Option<u8> is for tools that draft with a player-specified face
    DraftDie(usize, Option<u8>),
    UseTool(usize),
}

impl Tool {
    pub fn in_wrong_phase(&self, phase: TurnPhase) -> bool {
        matches!(
            (phase, self.tool_type),
            (TurnPhase::SelectTemplate, _)
                | (TurnPhase::GameOver, _)
                | (TurnPhase::FirstDraft, ToolType::RerollAllDiceInPool)
                | (TurnPhase::SecondDraft, ToolType::DraftTwoDice)
        )
    }
}
