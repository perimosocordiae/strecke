use std::collections::HashMap;

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
    #[serde(flatten)]
    view: PlayerView<'a>,
    turn: &'a TurnInfo,
    is_over: bool,
    is_winner: bool,
}

/// View of the game state for a specific player.
#[derive(Debug, Serialize)]
struct PlayerView<'a> {
    board: &'a board::Board,
    hand: Option<&'a [Tile]>,
    curr_player_idx: usize,
}

pub struct StreckeAPI {
    // Current game state
    state: GameManager,
    // List of all players in the game
    player_info: Vec<PlayerInfo>,
    // Player ID -> Agent mapping
    agents: HashMap<String, Box<dyn Agent + Send>>,
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
    fn view(&'_ self, player_id: &str) -> Result<PlayerView<'_>> {
        if self.game_over {
            return Ok(PlayerView {
                board: &self.state.board,
                hand: None,
                curr_player_idx: 0,
            });
        }
        let curr_player_id = &self.state.current_player().username;
        let curr_player_idx = self
            .player_info
            .iter()
            .position(|p| p.id == *curr_player_id)
            .ok_or("Invalid player ID")?;
        Ok(PlayerView {
            board: &self.state.board,
            hand: self.player_hand(player_id),
            curr_player_idx,
        })
    }
    fn do_action<F: FnMut(&str, &str)>(
        &mut self,
        action: &Action,
        mut notice_cb: F,
    ) -> Result<()> {
        let turn_info = TurnInfo {
            tile: self.state.current_player().tiles_in_hand[action.tile_idx],
            pos: self.state.current_player_pos().next_tile_position(),
            facing: action.facing,
        };
        self.game_over = self
            .state
            .take_turn(action.tile_idx, action.facing)
            .is_some();
        // Notify all human players of the action.
        for player_id in self.human_player_ids() {
            let view = self.view(player_id)?;
            let is_winner = self.game_over && view.hand.is_some();
            let msg = TakeTurnMessage {
                view,
                turn: &turn_info,
                is_over: self.game_over,
                is_winner,
            };
            let msg = serde_json::to_string(&msg)?;
            notice_cb(player_id, &msg);
        }
        Ok(())
    }
    fn human_player_ids(&self) -> impl Iterator<Item = &String> {
        self.player_info
            .iter()
            .filter(|p| p.level.is_none())
            .map(|p| &p.id)
    }
    fn process_agents<F: FnMut(&str, &str)>(
        &mut self,
        mut notice_cb: F,
    ) -> Result<()> {
        while !self.game_over
            && let Some(ai) = self.agents.get(self.current_player_id())
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
        let positions =
            (0..board::NOT_READY).choose_multiple(&mut rng, players.len());
        let mut agents = HashMap::new();
        for (player, edge_pos) in players.iter().zip(positions) {
            state.register_player(
                player.id.clone(),
                board::edge_position(edge_pos),
            )?;
            if player.level.is_some() {
                agents.insert(player.id.clone(), create_agent(0));
            }
        }
        Ok(Self {
            state,
            player_info: players.to_vec(),
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
        for player_id in self.human_player_ids() {
            notice_cb(player_id, &msg);
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
        let view = self.view(player_id)?;
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
