use crate::color::Color;
use crate::constants::{BOARD_COLS, BOARD_ROWS};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Slot {
    Any,
    Color(Color),
    Face(u8),
}
impl Display for Slot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Slot::Any => write!(f, "__"),
            Slot::Color(color) => write!(f, "{}_", color),
            Slot::Face(face) => write!(f, "_{}", face),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoardTemplate {
    pub slots: [[Slot; BOARD_COLS]; BOARD_ROWS],
    pub value: u8,
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
pub const ALL_BOARD_TEMPLATES: [[BoardTemplate; 2]; 12] = [
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
    [
        BoardTemplate {
            // Symphony of Light
            slots: [
                [DF2, ANY, DF5, ANY, DF1],
                [YEL, DF6, PPL, DF2, RED],
                [ANY, BLU, DF4, GRN, ANY],
                [ANY, DF3, ANY, DF5, ANY],
            ],
            value: 6,
        },
        BoardTemplate {
            // Virtus
            slots: [
                [DF4, ANY, DF2, DF5, GRN],
                [ANY, ANY, DF6, GRN, DF2],
                [ANY, DF3, GRN, DF4, ANY],
                [DF5, GRN, DF1, ANY, ANY],
            ],
            value: 5,
        },
    ],
    [
        BoardTemplate {
            // Aurorae Magnificus
            slots: [
                [DF5, GRN, BLU, PPL, DF2],
                [PPL, ANY, ANY, ANY, YEL],
                [YEL, ANY, DF6, ANY, PPL],
                [DF1, ANY, ANY, GRN, DF4],
            ],
            value: 5,
        },
        BoardTemplate {
            // Aurora Sagradis
            slots: [
                [RED, ANY, BLU, ANY, YEL],
                [DF4, PPL, DF3, GRN, DF2],
                [ANY, DF1, ANY, DF5, ANY],
                [ANY, ANY, DF6, ANY, ANY],
            ],
            value: 4,
        },
    ],
    [
        BoardTemplate {
            // Industria
            slots: [
                [DF1, RED, DF3, ANY, DF6],
                [DF5, DF4, RED, DF2, ANY],
                [ANY, ANY, DF5, RED, DF1],
                [ANY, ANY, ANY, DF3, RED],
            ],
            value: 5,
        },
        BoardTemplate {
            // Via Lux
            slots: [
                [YEL, ANY, DF6, ANY, ANY],
                [ANY, DF1, DF5, ANY, DF2],
                [DF3, YEL, RED, PPL, ANY],
                [ANY, ANY, DF4, DF3, RED],
            ],
            value: 4,
        },
    ],
    [
        BoardTemplate {
            // Sun's Glory
            slots: [
                [DF1, PPL, YEL, ANY, DF4],
                [PPL, YEL, ANY, ANY, DF6],
                [YEL, ANY, ANY, DF5, DF3],
                [ANY, DF5, DF4, DF2, DF1],
            ],
            value: 6,
        },
        BoardTemplate {
            // Firelight
            slots: [
                [DF3, DF4, DF1, DF5, ANY],
                [ANY, DF6, DF2, ANY, YEL],
                [ANY, ANY, ANY, YEL, RED],
                [DF5, ANY, YEL, RED, DF6],
            ],
            value: 5,
        },
    ],
    [
        BoardTemplate {
            // Lux Mundi
            slots: [
                [ANY, ANY, DF1, ANY, ANY],
                [DF1, GRN, DF3, BLU, DF2],
                [BLU, DF5, DF4, DF6, GRN],
                [ANY, BLU, DF5, GRN, ANY],
            ],
            value: 6,
        },
        BoardTemplate {
            // Lux Astram
            slots: [
                [ANY, DF1, GRN, PPL, DF4],
                [DF6, PPL, DF2, DF5, GRN],
                [DF1, GRN, DF5, DF3, PPL],
                [ANY, ANY, ANY, ANY, ANY],
            ],
            value: 5,
        },
    ],
    [
        BoardTemplate {
            // Water of Life
            slots: [
                [DF6, BLU, ANY, ANY, DF1],
                [ANY, DF5, BLU, ANY, ANY],
                [DF4, RED, DF2, BLU, ANY],
                [GRN, DF6, YEL, DF3, PPL],
            ],
            value: 6,
        },
        BoardTemplate {
            // Gravitas
            slots: [
                [DF1, ANY, DF3, BLU, ANY],
                [ANY, DF2, BLU, ANY, ANY],
                [DF6, BLU, ANY, DF4, ANY],
                [BLU, DF5, DF2, ANY, DF1],
            ],
            value: 5,
        },
    ],
    [
        BoardTemplate {
            // Firmitas
            slots: [
                [PPL, DF6, ANY, ANY, DF3],
                [DF5, PPL, DF3, ANY, ANY],
                [ANY, DF2, PPL, DF1, ANY],
                [ANY, DF1, DF5, PPL, DF4],
            ],
            value: 5,
        },
        BoardTemplate {
            // Kaleidoscopic Dream
            slots: [
                [YEL, BLU, ANY, ANY, DF1],
                [GRN, ANY, DF5, ANY, DF4],
                [DF3, ANY, RED, ANY, GRN],
                [DF2, ANY, ANY, BLU, YEL],
            ],
            value: 4,
        },
    ],
    [
        BoardTemplate {
            // Chromatic Splendor
            slots: [
                [ANY, ANY, GRN, ANY, ANY],
                [DF2, YEL, DF5, BLU, DF1],
                [ANY, RED, DF3, PPL, ANY],
                [DF1, ANY, DF6, ANY, DF4],
            ],
            value: 4,
        },
        BoardTemplate {
            // Comitas
            slots: [
                [YEL, ANY, DF2, ANY, DF6],
                [ANY, DF4, ANY, DF5, YEL],
                [ANY, ANY, ANY, YEL, DF5],
                [DF1, DF2, YEL, DF3, ANY],
            ],
            value: 5,
        },
    ],
];
