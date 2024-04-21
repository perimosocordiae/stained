use crate::constants::{BOARD_COLS, BOARD_ROWS};
use crate::game::{GameState, Player, TurnAction, TurnPhase};
use rand::prelude::IteratorRandom;

pub trait Agent {
    fn choose_action(&self, game: &GameState) -> TurnAction;
}

pub fn create_agent(_difficulty: usize) -> Box<dyn Agent + Send> {
    Box::<RandomAgent>::default()
}

#[derive(Default)]
struct RandomAgent;
impl Agent for RandomAgent {
    fn choose_action(&self, game: &GameState) -> TurnAction {
        let mut rng = rand::thread_rng();
        let me = game.current_player();
        match game.phase {
            TurnPhase::SelectTemplate => {
                let idx = (0..me.templates.len()).choose(&mut rng).unwrap();
                TurnAction { idx, coords: None }
            }
            TurnPhase::FirstDraft | TurnPhase::SecondDraft => {
                let choices = all_valid_drafts(game, me);
                if let Some(action) = choices.into_iter().choose(&mut rng) {
                    action
                } else {
                    TurnAction::default() // Pass
                }
            }
            TurnPhase::GameOver => TurnAction::default(),
        }
    }
}

fn all_valid_drafts(game: &GameState, player: &Player) -> Vec<TurnAction> {
    let mut valid_drafts = Vec::new();
    for (idx, die) in game.draft_pool.iter().enumerate() {
        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                if player.can_place_die((row, col), *die).is_none() {
                    valid_drafts.push(TurnAction {
                        idx,
                        coords: Some((row, col)),
                    });
                }
            }
        }
    }
    valid_drafts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_agent() {
        let mut game = GameState::init(2).unwrap();
        let agent = create_agent(0);
        // Select templates
        game.take_turn(agent.choose_action(&game)).unwrap();
        game.take_turn(agent.choose_action(&game)).unwrap();
        // Play 10 rounds, 2 drafts each, 2 players
        for _ in 0..(2 * 2 * 10) {
            game.take_turn(agent.choose_action(&game)).unwrap();
        }
        // Check that we are in the GameOver phase
        assert!(matches!(game.phase, TurnPhase::GameOver));
    }
}
