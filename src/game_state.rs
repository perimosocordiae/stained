use rand::{prelude::SliceRandom, seq::IteratorRandom};
use serde::{Deserialize, Serialize};

type DynError = Box<dyn std::error::Error>;

const NUM_ROUNDS: usize = 10;
const NUM_OBJECTIVES: usize = 3;
const NUM_TOOLS: usize = 3;
const MAX_PLAYERS: usize = 4;
const NUM_COLORS: usize = 5;
const DICE_PER_COLOR: usize = (2 * MAX_PLAYERS + 1) * NUM_ROUNDS / NUM_COLORS;
const BOARD_ROWS: usize = 4;
const BOARD_COLS: usize = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    players: Vec<Player>,
    start_player_idx: usize,
    curr_player_idx: usize,
    dice_bag: Vec<Color>,
    draft_pool: Vec<Dice>,
    round_track: Vec<Dice>,
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
            .choose_multiple(&mut rng, num_players * 2).collect();
        let players = ALL_COLORS
            .choose_multiple(&mut rng, num_players)
            .zip(player_templates.as_slice().chunks(2))
            .map(|(secret, templates)| Player {
                tokens: 0,
                board: [[BoardCell::Slot(Slot::Any); BOARD_COLS]; BOARD_ROWS],
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
    pub fn curr_player(&self) -> &Player {
        &self.players[self.curr_player_idx]
    }
    pub fn is_finished(&self) -> bool {
        self.round_track.len() >= NUM_ROUNDS && self.draft_pool.is_empty()
    }
    fn start_round(&mut self) {
        let num_dice: usize = 2 * self.players.len() + 1;
        self.draft_pool = self
            .dice_bag
            .split_off(self.dice_bag.len() - num_dice)
            .into_iter()
            .map(roll_die)
            .collect();
    }
    fn finish_round(&mut self) {
        assert_eq!(self.draft_pool.len(), 1);
        self.round_track.push(self.draft_pool.pop().unwrap());
        self.start_player_idx = (self.start_player_idx + 1) % self.players.len();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    tokens: u8,
    board: [[BoardCell; BOARD_COLS]; BOARD_ROWS],
    secret: Color,
    templates: Vec<BoardTemplate>,
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

pub fn roll_die(color: Color) -> Dice {
    let mut rng = rand::thread_rng();
    Dice {
        color,
        face: (1..=6).choose(&mut rng).unwrap_or(1),
    }
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
const ALL_COLORS: [Color; NUM_COLORS] = [
    Color::Red,
    Color::Yellow,
    Color::Green,
    Color::Blue,
    Color::Purple,
];

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardTemplate {
    slots: [[Slot; BOARD_COLS]; BOARD_ROWS],
    value: u8,
}
// TODO: move this to a data file.
const RED: Slot = Slot::Color(Color::Red);
const YEL: Slot = Slot::Color(Color::Yellow);
const GRN: Slot = Slot::Color(Color::Green);
const BLU: Slot = Slot::Color(Color::Blue);
const PPL: Slot = Slot::Color(Color::Purple);
const DF1: Slot = Slot::Face(1);
const DF2: Slot = Slot::Face(2);
const DF3: Slot = Slot::Face(3);
const DF4: Slot = Slot::Face(4);
const DF5: Slot = Slot::Face(5);
const DF6: Slot = Slot::Face(6);
const ANY: Slot = Slot::Any;
const ALL_BOARD_TEMPLATES: [[BoardTemplate; 2]; 4] = [
    [
        BoardTemplate {
            // Bellesguard
            slots: [
                [BLU, DF6, ANY, ANY, YEL],
                [ANY, DF3, BLU, ANY, ANY],
                [ANY, DF5, DF6, DF2, ANY],
                [ANY, DF4, ANY, DF1, GRN],
            ],
            value: 3,
        },
        BoardTemplate {
            // Batllo
            slots: [
                [ANY, ANY, DF6, ANY, ANY],
                [ANY, DF5, BLU, DF4, ANY],
                [DF3, GRN, YEL, PPL, DF2],
                [DF1, DF4, RED, DF5, DF3],
            ],
            value: 5,
        },
    ],
    [
        BoardTemplate {
            // Fractal Drops
            slots: [
                [ANY, DF4, ANY, YEL, DF6],
                [RED, ANY, DF2, ANY, ANY],
                [ANY, ANY, RED, PPL, DF1],
                [BLU, YEL, ANY, ANY, ANY],
            ],
            value: 3,
        },
        BoardTemplate {
            // Ripples of Light
            slots: [
                [ANY, ANY, ANY, RED, DF5],
                [ANY, ANY, PPL, DF4, BLU],
                [ANY, BLU, DF3, YEL, DF6],
                [YEL, DF2, GRN, DF1, RED],
            ],
            value: 5,
        },
    ],
    [
        BoardTemplate {
            // Luz Celestial
            slots: [
                [ANY, ANY, RED, DF5, ANY],
                [PPL, DF4, ANY, GRN, DF3],
                [DF6, ANY, ANY, BLU, ANY],
                [ANY, YEL, DF2, ANY, ANY],
            ],
            value: 3,
        },
        BoardTemplate {
            // Fulgor del Cielo
            slots: [
                [ANY, BLU, RED, ANY, ANY],
                [ANY, DF4, DF5, ANY, BLU],
                [BLU, DF2, ANY, RED, DF5],
                [DF6, RED, DF3, DF1, ANY],
            ],
            value: 5,
        },
    ],
    [
        BoardTemplate {
            // Sun Catcher
            slots: [
                [ANY, BLU, DF2, ANY, YEL],
                [ANY, DF4, ANY, RED, ANY],
                [ANY, ANY, DF5, YEL, ANY],
                [GRN, DF3, ANY, ANY, PPL],
            ],
            value: 3,
        },
        BoardTemplate {
            // Shadow Thief
            slots: [
                [DF6, PPL, ANY, ANY, DF5],
                [DF5, ANY, PPL, ANY, ANY],
                [RED, DF6, ANY, PPL, ANY],
                [YEL, RED, DF5, DF4, DF3],
            ],
            value: 5,
        },
    ],
];
