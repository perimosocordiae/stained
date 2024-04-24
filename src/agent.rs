use crate::constants::{BOARD_COLS, BOARD_ROWS};
use crate::game::{GameState, Player};
use crate::turn::{ActionType, TurnAction, TurnPhase};
use rand::prelude::{IteratorRandom, SliceRandom};

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
                let idx =
                    ActionType::SelectTemplate((0..me.templates.len()).choose(&mut rng).unwrap());
                TurnAction { idx, coords: None }
            }
            TurnPhase::FirstDraft | TurnPhase::SecondDraft => {
                if let Some(action) = all_valid_drafts(game, me).choose(&mut rng) {
                    action.clone()
                } else if let Some(action) = all_valid_tools(game, me).choose(&mut rng) {
                    action.clone()
                } else {
                    TurnAction::pass()
                }
            }
            TurnPhase::GameOver => TurnAction::pass(),
        }
    }
}

fn all_valid_drafts(game: &GameState, player: &Player) -> Vec<TurnAction> {
    let mut valid_drafts = Vec::new();
    for (idx, die) in game.draft_pool.iter().enumerate() {
        for row in 0..BOARD_ROWS {
            for col in 0..BOARD_COLS {
                if player.can_place_die((row, col), *die).is_ok() {
                    valid_drafts.push(TurnAction {
                        idx: ActionType::DraftDie(idx),
                        coords: Some((row, col)),
                    });
                }
            }
        }
    }
    valid_drafts
}

fn all_valid_tools(game: &GameState, player: &Player) -> Vec<TurnAction> {
    game.tools
        .iter()
        .enumerate()
        .filter_map(|(idx, tool)| {
            player.can_use_tool(tool).ok().map(|_| TurnAction {
                idx: ActionType::UseTool(idx),
                coords: None,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_agent() -> Result<(), Box<dyn std::error::Error>> {
        let mut game = GameState::init(2)?;
        let agent = create_agent(0);
        // Select templates
        game.take_turn(&agent.choose_action(&game))?;
        game.take_turn(&agent.choose_action(&game))?;
        // Play until game is finished
        while !game.is_finished() {
            game.take_turn(&agent.choose_action(&game))?;
        }
        // Check that we are in the GameOver phase
        assert!(matches!(game.phase, TurnPhase::GameOver));
        // Compute final scores
        let final_scores = game.player_scores();
        assert_eq!(final_scores.len(), 2, "{:?}", final_scores);
        Ok(())
    }
}
