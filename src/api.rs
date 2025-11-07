use blau_api::{DynSafeGameAPI, GameAPI, PlayerInfo, Result};
use serde::{Deserialize, Serialize};

use crate::{
    agent::{Agent, create_agent},
    game::GameState,
    turn::TurnAction,
};

/// View of the current game for a specific player.
#[derive(Serialize)]
struct PlayerView<'a> {
    game: GameState, // Redacted to avoid leaking secrets
    winner_id: Option<&'a str>,
}

/// Final data to store for viewing completed games.
#[derive(Serialize, Deserialize)]
struct FinalState {
    game: GameState,
    scores: Vec<i32>,
}

pub struct StainedAPI {
    // Current game state
    state: GameState,
    // Player IDs in the same order as agents
    player_ids: Vec<String>,
    // None if human player
    agents: Vec<Option<Box<dyn Agent + Send>>>,
    // Indicates if the game is over
    game_over: bool,
}

impl StainedAPI {
    fn view(&self, player_idx: usize) -> Result<String> {
        let mut game = self.state.clone();
        let winner_id = if self.game_over {
            let scores = game.player_scores();
            let max_score = *scores.iter().max().unwrap();
            let max_indices: Vec<usize> =
                scores
                    .iter()
                    .enumerate()
                    .filter_map(|(idx, &score)| {
                        if score == max_score { Some(idx) } else { None }
                    })
                    .collect();
            // TODO: Handle ties properly
            Some(self.player_ids[max_indices[0]].as_str())
        } else {
            game.redact_secrets(player_idx);
            None
        };
        Ok(serde_json::to_string(&PlayerView { game, winner_id })?)
    }
    fn do_action<F: FnMut(&str, &str)>(
        &mut self,
        action: &TurnAction,
        mut notice_cb: F,
    ) -> Result<()> {
        // Take the action.
        self.game_over = self.state.take_turn(action)?;
        // Notify all human players of the action.
        for idx in self.human_player_idxs() {
            notice_cb(self.player_ids[idx].as_str(), self.view(idx)?.as_str());
        }
        Ok(())
    }
    fn human_player_idxs(&self) -> impl Iterator<Item = usize> + '_ {
        self.agents.iter().enumerate().filter_map(|(idx, agent)| {
            if agent.is_none() { Some(idx) } else { None }
        })
    }
    fn process_agents<F: FnMut(&str, &str)>(
        &mut self,
        mut notice_cb: F,
    ) -> Result<()> {
        while !self.game_over
            && let Some(ai) = &self.agents[self.state.curr_player_idx]
        {
            let action = ai.choose_action(&self.state);
            self.do_action(&action, &mut notice_cb)?;
        }
        Ok(())
    }
}
impl GameAPI for StainedAPI {
    fn init(players: &[PlayerInfo], _params: Option<&str>) -> Result<Self> {
        let state = GameState::init(players.len())?;
        let player_ids = players.iter().map(|p| p.id.clone()).collect();
        let agents = players
            .iter()
            .map(|p| p.level.map(|lvl| create_agent(1 + lvl as usize)))
            .collect();
        Ok(Self {
            state,
            player_ids,
            agents,
            game_over: false,
        })
    }

    fn restore(player_info: &[PlayerInfo], final_state: &str) -> Result<Self> {
        let fs: FinalState = serde_json::from_str(final_state)?;
        Ok(Self {
            state: fs.game,
            player_ids: player_info.iter().map(|p| p.id.clone()).collect(),
            agents: vec![],
            game_over: true,
        })
    }

    fn start<F: FnMut(&str, &str)>(
        &mut self,
        game_id: i64,
        mut notice_cb: F,
    ) -> Result<()> {
        let msg = format!(r#"{{"action": "start", "game_id": {game_id}}}"#);
        for idx in self.human_player_idxs() {
            notice_cb(self.player_ids[idx].as_str(), &msg);
        }
        // Advance to wait for the next player action.
        self.process_agents(notice_cb)?;
        Ok(())
    }

    fn process_action<F: FnMut(&str, &str)>(
        &mut self,
        action: &str,
        mut notice_cb: F,
    ) -> Result<()> {
        if self.game_over {
            return Err("Game is over".into());
        }
        let action: TurnAction = serde_json::from_str(action)?;
        self.do_action(&action, &mut notice_cb)?;
        // Advance to wait for the next player action.
        self.process_agents(&mut notice_cb)?;
        Ok(())
    }
}

impl DynSafeGameAPI for StainedAPI {
    fn is_game_over(&self) -> bool {
        self.game_over
    }

    fn final_state(&self) -> Result<String> {
        if !self.game_over {
            return Err("Game is not finished".into());
        }
        let fs = FinalState {
            game: self.state.clone(),
            scores: self.state.player_scores(),
        };
        Ok(serde_json::to_string(&fs)?)
    }

    fn player_view(&self, player_id: &str) -> Result<String> {
        let player_idx = self
            .player_ids
            .iter()
            .position(|id| id == player_id)
            .ok_or("Unknown player ID")?;
        self.view(player_idx)
    }

    fn current_player_id(&self) -> &str {
        self.player_ids[self.state.curr_player_idx].as_str()
    }

    fn player_scores(&self) -> Vec<i32> {
        self.state.player_scores()
    }
}

#[test]
fn exercise_api() {
    let players = vec![
        PlayerInfo::human("foo".into()),
        PlayerInfo::human("bar".into()),
    ];
    let mut game: StainedAPI = GameAPI::init(&players, None).unwrap();
    game.start(1234, |id, msg| {
        assert!(id == "foo" || id == "bar");
        assert_eq!(msg, "{\"action\": \"start\", \"game_id\": 1234}");
    })
    .unwrap();

    let view_json = game.player_view("foo").unwrap();
    assert!(view_json.starts_with("{"));

    let mut num_notices = 0;
    game.process_action("{\"idx\": {\"SelectTemplate\": 0}}", |id, msg| {
        assert!(id == "foo" || id == "bar");
        assert!(msg.starts_with("{"));
        num_notices += 1;
    })
    .unwrap();
    // Should have notified both players of the action.
    assert_eq!(num_notices, 2);
}

#[test]
fn self_play() {
    let players = vec![
        PlayerInfo::ai("bot1".into(), 1),
        PlayerInfo::ai("bot2".into(), 1),
    ];
    let mut game: StainedAPI = GameAPI::init(&players, None).unwrap();
    // Run until game over
    game.start(1234, |_, _| {}).unwrap();
    assert!(game.is_game_over());
    // Smoke test the final_state method.
    let final_state = game.final_state().unwrap();
    assert!(final_state.starts_with("{"));
}
