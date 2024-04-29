use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub tool_type: ToolType,
    pub cost: u8,
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
// TODO: Uncomment tools as they are implemented.
pub const ALL_TOOL_TYPES: [ToolType; 6] = [
    ToolType::BumpDraftedDie,
    ToolType::FlipDraftedDie,
    ToolType::RerollDraftedDie,
    ToolType::SwapDraftedDieWithRoundTrack,
    // ToolType::SwapDraftedDieWithBag,
    ToolType::RerollAllDiceInPool,
    // ToolType::MoveDieIgnoringColor,
    // ToolType::MoveDieIgnoringValue,
    // ToolType::MoveExactlyTwoDice,
    // ToolType::MoveUpToTwoDiceMatchingColor,
    // ToolType::DraftTwoDice,
    ToolType::PlaceIgnoringAdjacency,
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolData {
    BumpDraftedDie {
        draft_idx: usize,
        is_increment: bool,
    },
    FlipDraftedDie {
        draft_idx: usize,
    },
    RerollDraftedDie {
        draft_idx: usize,
    },
    SwapDraftedDieWithRoundTrack {
        draft_idx: usize,
        round_idx: (usize, usize),
    },
    SwapDraftedDieWithBag {
        draft_idx: usize,
        face: u8,
    },
    RerollAllDiceInPool,
    MoveDieIgnoringColor {
        from: (usize, usize),
    },
    MoveDieIgnoringValue {
        from: (usize, usize),
    },
    MoveExactlyTwoDice {
        from: [(usize, usize); 2],
        to: [(usize, usize); 2],
    },
    MoveUpToTwoDiceMatchingColor {
        from: [(usize, usize); 2],
        to: [(usize, usize); 2],
        round_idx: (usize, usize),
    },
    DraftTwoDice,
    PlaceIgnoringAdjacency,
}
impl ToolData {
    pub fn matches_type(&self, tool_type: ToolType) -> bool {
        matches!(
            (self, tool_type),
            (Self::BumpDraftedDie { .. }, ToolType::BumpDraftedDie)
                | (Self::FlipDraftedDie { .. }, ToolType::FlipDraftedDie)
                | (Self::RerollDraftedDie { .. }, ToolType::RerollDraftedDie)
                | (
                    Self::SwapDraftedDieWithRoundTrack { .. },
                    ToolType::SwapDraftedDieWithRoundTrack
                )
                | (
                    Self::SwapDraftedDieWithBag { .. },
                    ToolType::SwapDraftedDieWithBag
                )
                | (Self::RerollAllDiceInPool, ToolType::RerollAllDiceInPool)
                | (
                    Self::MoveDieIgnoringColor { .. },
                    ToolType::MoveDieIgnoringColor
                )
                | (
                    Self::MoveDieIgnoringValue { .. },
                    ToolType::MoveDieIgnoringValue
                )
                | (
                    Self::MoveExactlyTwoDice { .. },
                    ToolType::MoveExactlyTwoDice
                )
                | (
                    Self::MoveUpToTwoDiceMatchingColor { .. },
                    ToolType::MoveUpToTwoDiceMatchingColor
                )
                | (Self::DraftTwoDice, ToolType::DraftTwoDice)
                | (
                    Self::PlaceIgnoringAdjacency,
                    ToolType::PlaceIgnoringAdjacency
                )
        )
    }
}
