use crate::board::BoardCell;
use crate::color::{Color, Dice, ALL_COLORS};
use crate::constants::*;
use crate::objective::{Objective, ALL_OBJECTIVES};
use crate::template::{BoardTemplate, Slot, ALL_BOARD_TEMPLATES};
use crate::tool::{Tool, ToolData, ToolType, ALL_TOOL_TYPES};
use crate::turn::{ActionType, TurnAction, TurnPhase};
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
    pub round_track: Vec<Vec<Dice>>,
    pub tools: Vec<Tool>,
    objectives: Vec<Objective>,
}
impl GameState {
    pub fn init(num_players: usize) -> Result<Self, DynError> {
        if !(2..=MAX_PLAYERS).contains(&num_players) {
            return Err("Invalid number of players".into());
        }
        let mut dice_bag = Vec::with_capacity(DICE_PER_COLOR * NUM_COLORS);
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
                active_tool: None,
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
    pub fn is_finished(&self) -> bool {
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
    pub fn take_turn(&mut self, action: &TurnAction) -> Result<bool, DynError> {
        println!(
            "{:?} (P{}) => {:?}",
            self.phase, self.curr_player_idx, action
        );
        match self.phase {
            TurnPhase::SelectTemplate => {
                if let ActionType::SelectTemplate(idx) = action.idx {
                    self.players[self.curr_player_idx].select_template(idx)?;
                } else {
                    return Err("Invalid action: must select a template".into());
                }
                self.curr_player_idx = self.next_idx(self.curr_player_idx);
                if self.curr_player_idx == self.start_player_idx {
                    self.start_round();
                }
            }
            TurnPhase::FirstDraft => {
                if self.handle_action(action)? {
                    self.curr_player_idx = self.next_idx(self.curr_player_idx);
                    if self.curr_player_idx == self.start_player_idx {
                        self.curr_player_idx = self.prev_idx(self.curr_player_idx);
                        self.phase = TurnPhase::SecondDraft;
                    }
                }
            }
            TurnPhase::SecondDraft => {
                if self.handle_action(action)? {
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
            }
            TurnPhase::GameOver => return Err("Game is over".into()),
        }
        Ok(matches!(self.phase, TurnPhase::GameOver))
    }
    fn handle_action(&mut self, action: &TurnAction) -> Result<bool, DynError> {
        match action.idx {
            ActionType::SelectTemplate(_) => {
                Err("Invalid action: templates have already been selected".into())
            }
            ActionType::DraftDie(idx) => {
                if let Some(coords) = action.coords {
                    self.draft_pool.get(idx).ok_or("Invalid die index")?;
                    let die = self.draft_pool.remove(idx);
                    self.players[self.curr_player_idx].place_die(coords, die)?;
                }
                self.players[self.curr_player_idx].active_tool = None;
                Ok(true)
            }
            ActionType::UseTool(idx) => {
                let tool = self.tools.get(idx).ok_or("Invalid tool index")?;
                let data = action.tool.as_ref().ok_or("Tool action missing data")?;
                let mut rng = rand::thread_rng();
                match data {
                    ToolData::RerollAllDiceInPool => {
                        self.draft_pool
                            .iter_mut()
                            .for_each(|die| die.reroll(&mut rng));
                    }
                    ToolData::PlaceIgnoringAdjacency => {
                        self.players[self.curr_player_idx].active_tool = Some(tool.tool_type);
                    }
                    ToolData::FlipDraftedDie { draft_idx } => {
                        self.draft_pool
                            .get_mut(*draft_idx)
                            .ok_or("Invalid draft index")?
                            .flip();
                    }
                    ToolData::RerollDraftedDie { draft_idx } => {
                        self.draft_pool
                            .get_mut(*draft_idx)
                            .ok_or("Invalid draft index")?
                            .reroll(&mut rng);
                    }
                    ToolData::BumpDraftedDie {
                        draft_idx,
                        is_increment,
                    } => {
                        let face = self
                            .draft_pool
                            .get(*draft_idx)
                            .ok_or("Invalid draft index")?
                            .face;
                        match (*is_increment, face) {
                            (true, 6) => {
                                return Err("Cannot increment a die past 6".into());
                            }
                            (false, 1) => {
                                return Err("Cannot decrement a die below 1".into());
                            }
                            (true, _) => self.draft_pool[*draft_idx].increment(),
                            (false, _) => self.draft_pool[*draft_idx].decrement(),
                        }
                    }
                    ToolData::SwapDraftedDieWithRoundTrack {
                        draft_idx,
                        round_idx,
                    } => {
                        let src = self
                            .round_track
                            .get_mut(round_idx.0)
                            .ok_or("Invalid round index")?
                            .get_mut(round_idx.1)
                            .ok_or("Invalid die index")?;
                        let dst = self
                            .draft_pool
                            .get_mut(*draft_idx)
                            .ok_or("Invalid draft pool index")?;
                        std::mem::swap(src, dst);
                    }
                    _ => todo!("Implement tool: {t:?}", t = tool.tool_type),
                }
                self.players[self.curr_player_idx].tokens -= tool.cost;
                if tool.cost == 1 {
                    self.tools[idx].cost = 2;
                }
                Ok(false)
            }
        }
    }
    fn start_round(&mut self) {
        let mut rng = rand::thread_rng();
        self.draft_pool = self
            .dice_bag
            .split_off(self.dice_bag.len() - self.pool_size())
            .into_iter()
            .map(|color| Dice::roll(color, &mut rng))
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
pub struct Player {
    tokens: u8,
    board: [[BoardCell; BOARD_COLS]; BOARD_ROWS],
    secret: Color,
    pub templates: Vec<BoardTemplate>,
    active_tool: Option<ToolType>,
}
impl Player {
    fn select_template(&mut self, idx: usize) -> Result<(), DynError> {
        let template = self.templates.get(idx).ok_or("Invalid template index")?;
        self.tokens = template.value;
        for i in 0..BOARD_ROWS {
            for j in 0..BOARD_COLS {
                self.board[i][j].slot = template.slots[i][j];
            }
        }
        Ok(())
    }
    pub fn can_place_die(&self, coords: (usize, usize), die: Dice) -> Result<(), DynError> {
        let row = self.board.get(coords.0).ok_or("Invalid row")?;
        let cell = row.get(coords.1).ok_or("Invalid column")?;
        if cell.die.is_some() {
            return Err("Cell is already occupied".into());
        }
        match cell.slot {
            Slot::Color(color) if color != die.color => {
                return Err("Die color does not match slot".into());
            }
            Slot::Face(face) if face != die.face => {
                return Err("Die face does not match slot".into());
            }
            _ => {}
        }
        // Check orthogonally adjacent cells.
        let nbr_dice: Vec<Dice> = neighbor_coords(coords)
            .filter_map(|(r, c)| self.board[r][c].die)
            .collect();
        for ndr_die in nbr_dice.iter() {
            if die.color == ndr_die.color {
                return Err("Die color matches orthogonally adjacent die".into());
            } else if die.face == ndr_die.face {
                return Err("Die face matches orthogonally adjacent die".into());
            }
        }
        // Check diagonally adjacent cells if we don't have any orthogonally adjacent dice.
        if nbr_dice.is_empty()
            && !matches!(self.active_tool, Some(ToolType::PlaceIgnoringAdjacency))
            && !diagonal_coords(coords).any(|(r, c)| self.board[r][c].die.is_some())
        {
            if self.board.iter().flatten().any(|cell| cell.die.is_some()) {
                return Err("Die must be placed adjacent to another die".into());
            }
            if (1..BOARD_ROWS - 1).contains(&coords.0) && (1..BOARD_COLS - 1).contains(&coords.1) {
                return Err("First die must be placed on the edge".into());
            }
        }
        Ok(())
    }
    fn place_die(&mut self, coords: (usize, usize), mut die: Dice) -> Result<(), DynError> {
        match self.active_tool {
            Some(ToolType::FlipDraftedDie) => die.flip(),
            Some(ToolType::RerollDraftedDie) => die.reroll(&mut rand::thread_rng()),
            _ => {}
        }
        self.can_place_die(coords, die)?;
        self.board[coords.0][coords.1].die = Some(die);
        Ok(())
    }
    pub fn can_use_tool(&self, tool: &Tool) -> Result<(), DynError> {
        if tool.cost > self.tokens {
            return Err("Insufficient tokens to use tool".into());
        }
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
        // Add one point per token.
        score += self.tokens as i32;
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
