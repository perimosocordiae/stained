use crate::turn::TurnPhase;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub tool_type: ToolType,
    pub cost: u8,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ToolType {
    // Tools that modify the draft pool.
    BumpDraftedDie, // +/- 1
    FlipDraftedDie, // 1 <-> 6, 2 <-> 5, 3 <-> 4
    RerollDraftedDie,
    SwapDraftedDieWithRoundTrack,
    SwapDraftedDieWithBag,
    RerollAllDiceInPool, // only before second draft
    // Tools that move dice already on the board.
    MoveDieIgnoringColor,
    MoveDieIgnoringValue,
    MoveExactlyTwoDice,
    MoveUpToTwoDiceMatchingColor, // must match a color on the round track
    // Tools that break a rule.
    DraftTwoDice, // only before first draft, skips second draft
    PlaceIgnoringAdjacency,
}
pub const ALL_TOOL_TYPES: [ToolType; 12] = [
    ToolType::BumpDraftedDie,
    ToolType::FlipDraftedDie,
    ToolType::RerollDraftedDie,
    ToolType::SwapDraftedDieWithRoundTrack,
    ToolType::SwapDraftedDieWithBag,
    ToolType::RerollAllDiceInPool,
    ToolType::MoveDieIgnoringColor,
    ToolType::MoveDieIgnoringValue,
    ToolType::MoveExactlyTwoDice,
    ToolType::MoveUpToTwoDiceMatchingColor,
    ToolType::DraftTwoDice,
    ToolType::PlaceIgnoringAdjacency,
];
