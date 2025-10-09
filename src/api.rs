use blau_api::{DynSafeGameAPI, GameAPI, PlayerInfo, Result};
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

use crate::{
    agent::{Agent, create_agent},
    board,
    game::GameManager,
    tiles::{Direction, Tile},
};

/// An action taken by a player.
#[derive(Debug, Serialize, Deserialize)]
struct Action {
    tile_idx: usize,
    facing: Direction,
}

/// Generic information about a turn that was taken.
#[derive(Debug, Serialize)]
struct TurnInfo {
    tile: Tile,
    pos: board::Position,
    facing: Direction,
}

/// Message sent to players after a turn is taken.
#[derive(Debug, Serialize)]
struct TakeTurnMessage<'a> {
    board: &'a board::Board,
    hand: Option<&'a [Tile]>,
    turn: &'a TurnInfo,
    is_over: bool,
    is_winner: bool,
}

/// View of the game state for a specific player.
#[derive(Serialize)]
struct PlayerView<'a> {
    board: &'a board::Board,
    players: &'a [String],
    hand: Option<&'a [Tile]>,
}

pub struct StreckeAPI {
    // Current game state
    state: GameManager,
    // Player IDs in the same order as agents
    player_ids: Vec<String>,
    // None if human player
    agents: Vec<Option<Box<dyn Agent + Send>>>,
    // Indicates if the game is over
    game_over: bool,
}

impl StreckeAPI {
    fn player_hand(&self, player_id: &str) -> Option<&[Tile]> {
        self.state.alive_players.iter().find_map(|p| {
            if p.username == player_id {
                Some(p.tiles_in_hand.as_slice())
            } else {
                None
            }
        })
    }
    fn do_action<F: FnMut(&str, &str)>(
        &mut self,
        action: &Action,
        mut notice_cb: F,
    ) -> Result<()> {
        self.game_over = self
            .state
            .take_turn(action.tile_idx, action.facing)
            .is_some();
        // Notify all human players of the action.
        let turn_info = TurnInfo {
            tile: self.state.current_player().tiles_in_hand[action.tile_idx],
            pos: self.state.current_player_pos().next_tile_position(),
            facing: action.facing,
        };
        for idx in self.human_player_idxs() {
            let player_id = self.player_ids[idx].as_str();
            let is_winner =
                self.game_over && self.state.get_player(player_id).is_some();
            let msg = TakeTurnMessage {
                board: &self.state.board,
                hand: self.player_hand(player_id),
                turn: &turn_info,
                is_over: self.game_over,
                is_winner,
            };
            let msg = serde_json::to_string(&msg)?;
            notice_cb(player_id, &msg);
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
            && let Some(ai) = &self.agents[self.state.current_player_idx]
        {
            let (tile_idx, facing) = ai.choose_action(&self.state);
            self.do_action(&Action { tile_idx, facing }, &mut notice_cb)?;
        }
        Ok(())
    }
}
impl GameAPI for StreckeAPI {
    fn init(players: &[PlayerInfo], _params: Option<&str>) -> Result<Self> {
        let mut rng = rand::rng();
        let mut state = GameManager::new(&mut rng);
        let player_ids =
            players.iter().map(|p| p.id.clone()).collect::<Vec<_>>();
        let positions =
            (0..board::NOT_READY).choose_multiple(&mut rng, player_ids.len());
        for (player_id, edge_pos) in player_ids.iter().zip(positions) {
            state.register_player(
                player_id.to_owned(),
                board::edge_position(edge_pos),
            )?;
        }

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
        let board: board::Board = serde_json::from_str(final_state)?;
        let mut res = Self::init(player_info, None)?;
        res.game_over = true;
        res.state.board = board;
        Ok(res)
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
        let action: Action = serde_json::from_str(action)?;
        self.do_action(&action, &mut notice_cb)?;
        // Advance to wait for the next player action.
        self.process_agents(&mut notice_cb)?;
        Ok(())
    }
}

impl DynSafeGameAPI for StreckeAPI {
    fn is_game_over(&self) -> bool {
        self.game_over
    }

    fn final_state(&self) -> Result<String> {
        if !self.game_over {
            return Err("Game is not finished".into());
        }
        Ok(serde_json::to_string(&self.state.board)?)
    }

    fn player_view(&self, player_id: &str) -> Result<String> {
        let view = PlayerView {
            board: &self.state.board,
            players: self.player_ids.as_slice(),
            hand: self.player_hand(player_id),
        };
        Ok(serde_json::to_string(&view)?)
    }

    fn current_player_id(&self) -> &str {
        &self.state.current_player().username
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
    let mut game: StreckeAPI = GameAPI::init(&players, None).unwrap();
    game.start(1234, |id, msg| {
        assert_eq!(id, "foo");
        assert_eq!(msg, "{\"action\": \"start\", \"game_id\": 1234}");
    })
    .unwrap();

    let view_json = game.player_view("foo").unwrap();
    assert!(view_json.starts_with("{"));

    let mut num_notices = 0;
    game.process_action(r#"{"tile_idx": 1, "facing": "West"}"#, |id, msg| {
        assert_eq!(id, "foo");
        assert!(msg.starts_with("{"));
        num_notices += 1;
    })
    .unwrap();
    if num_notices == 1 {
        // This means our move ended the game.
        assert!(game.is_game_over());
    } else {
        assert_eq!(num_notices, 2);
    }
}
