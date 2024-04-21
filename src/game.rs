use std::fmt::Display;

use crate::constants::*;
use crate::template::{Color, Slot, BoardTemplate, ALL_BOARD_TEMPLATES, ALL_COLORS};
use rand::{prelude::SliceRandom, seq::IteratorRandom};
use serde::{Deserialize, Serialize};

type DynError = Box<dyn std::error::Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    players: Vec<Player>,
    start_player_idx: usize,
    curr_player_idx: usize,
    pub phase: TurnPhase,
    dice_bag: Vec<Color>,
    pub draft_pool: Vec<Dice>,
    round_track: Vec<Vec<Dice>>,
    tools: Vec<Tool>,
    objectives: Vec<Objective>,
}
impl GameState {
    pub fn init(num_players: usize) -> Result<Self, DynError> {
        if !(2..=4).contains(&num_players) {
            return Err("Invalid number of players".into());
        }
        let mut dice_bag = Vec::with_capacity(DICE_PER_COLOR * 5);
        for _ in 0..DICE_PER_COLOR {
            dice_bag.extend_from_slice(ALL_COLORS.as_slice());
        }
        let mut rng = rand::thread_rng();
        dice_bag.shuffle(&mut rng);

        let start_player_idx = (0..num_players).choose(&mut rng).unwrap_or(0);
        let player_templates: Vec<_> = ALL_BOARD_TEMPLATES
            .choose_multiple(&mut rng, num_players * 2)
            .collect();
        let players = ALL_COLORS
            .choose_multiple(&mut rng, num_players)
            .zip(player_templates.as_slice().chunks(2))
            .map(|(secret, templates)| Player {
                tokens: 0,
                board: [[BoardCell::default(); BOARD_COLS]; BOARD_ROWS],
                secret: *secret,
                templates: templates.iter().flat_map(|x| x.iter().cloned()).collect(),
            })
            .collect();

        let tools = ALL_TOOL_TYPES
            .choose_multiple(&mut rng, NUM_TOOLS)
            .map(|tool_type| Tool {
                tool_type: *tool_type,
                cost: 1,
            })
            .collect();

        Ok(Self {
            players,
            start_player_idx,
            curr_player_idx: start_player_idx,
            phase: TurnPhase::SelectTemplate,
            dice_bag,
            draft_pool: Vec::new(),
            round_track: Vec::new(),
            tools,
            objectives: ALL_OBJECTIVES
                .choose_multiple(&mut rng, NUM_OBJECTIVES)
                .copied()
                .collect(),
        })
    }
    fn is_finished(&self) -> bool {
        self.round_track.len() >= NUM_ROUNDS && self.draft_pool.is_empty()
    }
    fn next_idx(&self, idx: usize) -> usize {
        (idx + 1) % self.players.len()
    }
    fn prev_idx(&self, idx: usize) -> usize {
        (idx + self.players.len() - 1) % self.players.len()
    }
    fn pool_size(&self) -> usize {
        2 * self.players.len() + 1
    }
    pub fn current_player(&self) -> &Player {
        &self.players[self.curr_player_idx]
    }
    pub fn take_turn(&mut self, action: TurnAction) -> Result<bool, DynError> {
        match self.phase {
            TurnPhase::SelectTemplate => {
                self.players[self.curr_player_idx].select_template(action.idx)?;
                self.curr_player_idx = self.next_idx(self.curr_player_idx);
                if self.curr_player_idx == self.start_player_idx {
                    self.start_round();
                }
            }
            TurnPhase::FirstDraft => {
                if let Some(coords) = action.coords {
                    let die = self.draft_pool.remove(action.idx);
                    self.players[self.curr_player_idx].place_die(coords, die)?;
                }
                self.curr_player_idx = self.next_idx(self.curr_player_idx);
                if self.curr_player_idx == self.start_player_idx {
                    self.curr_player_idx = self.prev_idx(self.curr_player_idx);
                    self.phase = TurnPhase::SecondDraft;
                }
            }
            TurnPhase::SecondDraft => {
                if let Some(coords) = action.coords {
                    let die = self.draft_pool.remove(action.idx);
                    self.players[self.curr_player_idx].place_die(coords, die)?;
                }
                if self.curr_player_idx == self.start_player_idx {
                    self.finish_round();
                    if self.is_finished() {
                        self.phase = TurnPhase::GameOver;
                    } else {
                        self.start_round();
                    }
                } else {
                    self.curr_player_idx = self.prev_idx(self.curr_player_idx);
                }
            }
            TurnPhase::GameOver => return Err("Game is over".into()),
        }
        Ok(matches!(self.phase, TurnPhase::GameOver))
    }
    fn start_round(&mut self) {
        self.draft_pool = self
            .dice_bag
            .split_off(self.dice_bag.len() - self.pool_size())
            .into_iter()
            .map(roll_die)
            .collect();
        self.phase = TurnPhase::FirstDraft;
    }
    fn finish_round(&mut self) {
        // Any remaining dice in the draft pool are moved to the round track.
        self.round_track.push(self.draft_pool.drain(..).collect());
        self.start_player_idx = self.next_idx(self.start_player_idx);
        self.curr_player_idx = self.start_player_idx;
    }
    pub fn player_scores(&self) -> Vec<i32> {
        self.players
            .iter()
            .map(|player| player.calculate_score(&self.objectives))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TurnPhase {
    SelectTemplate,
    FirstDraft,
    SecondDraft,
    GameOver,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TurnAction {
    pub idx: usize,
    pub coords: Option<(usize, usize)>,
    // TODO: tool selection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub tokens: u8,
    pub board: [[BoardCell; BOARD_COLS]; BOARD_ROWS],
    pub secret: Color,
    pub templates: Vec<BoardTemplate>,
}
impl Player {
    fn select_template(&mut self, idx: usize) -> Result<(), DynError> {
        if !(0..self.templates.len()).contains(&idx) {
            return Err("Invalid template index".into());
        }
        let template = &self.templates[idx];
        self.tokens = template.value;
        for i in 0..BOARD_ROWS {
            for j in 0..BOARD_COLS {
                self.board[i][j].slot = template.slots[i][j];
            }
        }
        Ok(())
    }
    pub fn can_place_die(&self, coords: (usize, usize), die: Dice) -> Option<&str> {
        if !(0..BOARD_ROWS).contains(&coords.0) || !(0..BOARD_COLS).contains(&coords.1) {
            return Some("Invalid coordinates");
        }
        let cell = &self.board[coords.0][coords.1];
        if cell.die.is_some() {
            return Some("Cell is already occupied");
        }
        match cell.slot {
            Slot::Color(color) if color != die.color => {
                return Some("Die color does not match slot");
            }
            Slot::Face(face) if face != die.face => {
                return Some("Die face does not match slot");
            }
            _ => {}
        }
        // Check orthogonally adjacent cells.
        let nbr_dice: Vec<Dice> = neighbor_coords(coords)
            .filter_map(|(r, c)| self.board[r][c].die)
            .collect();
        for ndr_die in nbr_dice.iter() {
            if die.color == ndr_die.color {
                return Some("Die color matches orthogonally adjacent die");
            } else if die.face == ndr_die.face {
                return Some("Die face matches orthogonally adjacent die");
            }
        }
        // Check diagonally adjacent cells if we don't have any orthogonally adjacent dice.
        if nbr_dice.is_empty()
            && diagonal_coords(coords).any(|(r, c)| self.board[r][c].die.is_some())
        {
            if self.board.iter().flatten().any(|cell| cell.die.is_some()) {
                return Some("Die must be placed adjacent to another die");
            }
            if (1..BOARD_ROWS - 1).contains(&coords.0) && (1..BOARD_COLS - 1).contains(&coords.1) {
                return Some("First die must be placed on the edge");
            }
        }
        None
    }
    fn place_die(&mut self, coords: (usize, usize), die: Dice) -> Result<(), DynError> {
        if let Some(msg) = self.can_place_die(coords, die) {
            return Err(msg.into());
        }
        self.board[coords.0][coords.1].die = Some(die);
        Ok(())
    }
    fn calculate_score(&self, objectives: &[Objective]) -> i32 {
        // One point for each die matching our secret color, and minus one
        // point for each slot without a die in it.
        let mut score = self
            .board
            .iter()
            .flatten()
            .map(|cell| match cell.die {
                Some(die) if die.color == self.secret => 1,
                None => -1,
                _ => 0,
            })
            .sum::<i32>();
        // Add the scores for the objectives.
        for obj in objectives.iter() {
            score += obj.score(&self.board);
        }
        score
    }
}

fn neighbor_coords(coords: (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
    let (r, c) = coords;
    [
        (r, c.wrapping_sub(1)),
        (r, c + 1),
        (r.wrapping_sub(1), c),
        (r + 1, c),
    ]
    .into_iter()
    .filter(|(r, c)| *r < BOARD_ROWS && *c < BOARD_COLS)
}

fn diagonal_coords(coords: (usize, usize)) -> impl Iterator<Item = (usize, usize)> {
    let (r, c) = coords;
    [
        (r.wrapping_sub(1), c.wrapping_sub(1)),
        (r.wrapping_sub(1), c + 1),
        (r + 1, c.wrapping_sub(1)),
        (r + 1, c + 1),
    ]
    .into_iter()
    .filter(|(r, c)| *r < BOARD_ROWS && *c < BOARD_COLS)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoardCell {
    slot: Slot,
    die: Option<Dice>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Dice {
    color: Color,
    face: u8,
}
impl Display for Dice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.color, self.face)
    }
}

pub fn roll_die(color: Color) -> Dice {
    let mut rng = rand::thread_rng();
    Dice {
        color,
        face: (1..=6).choose(&mut rng).unwrap_or(1),
    }
}

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
const ALL_OBJECTIVES: [Objective; 10] = [
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    tool_type: ToolType,
    cost: u8,
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
const ALL_TOOL_TYPES: [ToolType; 12] = [
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
