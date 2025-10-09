use blau_api::{GameAPI, PlayerInfo, Result};
use serde::Serialize;

use crate::{
    agent::{Agent, create_agent},
    game::GameState,
    turn::TurnAction,
};

#[derive(Serialize)]
struct TakeTurnMessage<'a> {
    game_data: &'a GameState,
    is_over: bool,
    winner_id: Option<&'a str>,
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
    fn winner_id(&self) -> Option<&str> {
        if !self.game_over {
            return None;
        }
        let scores = self.state.player_scores();
        let max_score = *scores.iter().max()?;
        let max_indices = scores
            .iter()
            .enumerate()
            .filter_map(|(idx, &score)| if score == max_score { Some(idx) } else { None })
            .collect::<Vec<_>>();
        // TODO: Handle ties properly.
        Some(self.player_ids[max_indices[0]].as_str())
    }
    fn do_action<F: FnMut(&str, &str)>(
        &mut self,
        action: &TurnAction,
        mut notice_cb: F,
    ) -> Result<()> {
        self.game_over = self.state.take_turn(action)?;
        // Notify all human players of the action.
        let msg = TakeTurnMessage {
            game_data: &self.state,
            is_over: self.game_over,
            winner_id: self.winner_id(),
        };
        let msg = serde_json::to_string(&msg)?;
        for idx in self.human_player_idxs() {
            notice_cb(self.player_ids[idx].as_str(), &msg);
        }
        Ok(())
    }
    fn human_player_idxs(&self) -> impl Iterator<Item = usize> + '_ {
        self.agents.iter().enumerate().filter_map(
            |(idx, agent)| {
                if agent.is_none() { Some(idx) } else { None }
            },
        )
    }
    fn process_agents<F: FnMut(&str, &str)>(&mut self, mut notice_cb: F) -> Result<()> {
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
            .map(|p| p.level.map(|_| create_agent(0)))
            .collect();
        Ok(Self {
            state,
            player_ids,
            agents,
            game_over: false,
        })
    }

    fn restore(player_info: &[PlayerInfo], final_state: &str) -> Result<Self> {
        let state: GameState = serde_json::from_str(final_state)?;
        Ok(Self {
            state,
            player_ids: player_info.iter().map(|p| p.id.clone()).collect(),
            agents: vec![],
            game_over: true,
        })
    }

    fn is_game_over(&self) -> bool {
        self.game_over
    }

    fn final_state(&self) -> Result<String> {
        if !self.game_over {
            return Err("Game is not finished".into());
        }
        Ok(serde_json::to_string(&self.state)?)
    }

    fn player_view(&self, player_id: &str) -> Result<String> {
        let player_idx = self
            .player_ids
            .iter()
            .position(|id| id == player_id)
            .ok_or("Unknown player ID")?;
        let mut game = self.state.clone();
        game.redact_secrets(player_idx);
        Ok(serde_json::to_string(&game)?)
    }

    fn start<F: FnMut(&str, &str)>(&mut self, game_id: i64, mut notice_cb: F) -> Result<()> {
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
        PlayerInfo::ai("bot".into(), 1),
    ];
    let mut game: StainedAPI =
        GameAPI::init(&players, Some(r#"{"named_layout": "first"}"#)).unwrap();
    let mut num_notices = 0;
    game.start(1234, |id, msg| {
        assert_eq!(id, "foo");
        if num_notices == 0 {
            assert_eq!(msg, "{\"action\": \"start\", \"game_id\": 1234}");
        } else {
            assert!(msg.starts_with("{"));
        }
        num_notices += 1;
    })
    .unwrap();

    let view_json = game.player_view("foo").unwrap();
    assert!(view_json.starts_with("{"));

    let mut num_notices = 0;
    game.process_action(r#"{"idx": {"SelectTemplate": 0}}"#, |id, msg| {
        assert_eq!(id, "foo");
        assert!(msg.starts_with("{"));
        num_notices += 1;
    })
    .unwrap();
    // One for foo, one for bot's turn.
    assert_eq!(num_notices, 2);
}
