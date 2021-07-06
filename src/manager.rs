use crate::board::{Board, Position};
use crate::tiles::{all_tiles, Tile};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::cmp;

const TILES_PER_PLAYER: i32 = 3;

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    username: String,
    board_index: usize,
    tiles_in_hand: Vec<Tile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerMove {
    pub tile_index: usize,
}

pub struct GameManager {
    pub board: Board,
    tile_stack: Vec<Tile>,
    players: Vec<Player>,
    current_player_idx: usize,
}

impl GameManager {
    pub fn new(rng: &mut impl rand::Rng) -> Self {
        let mut tile_stack = all_tiles();
        tile_stack.shuffle(rng);
        GameManager {
            board: Board::new(),
            tile_stack,
            players: Vec::new(),
            current_player_idx: 0,
        }
    }
    pub fn register_player(
        &mut self,
        username: &str,
        start_position: Position,
    ) -> Result<(), String> {
        let pos = cmp::max(0, self.tile_stack.len() as i32 - TILES_PER_PLAYER)
            as usize;
        self.players.push(Player {
            username: username.to_owned(),
            board_index: self.board.add_player(start_position)?,
            tiles_in_hand: self.tile_stack.split_off(pos),
        });
        Ok(())
    }
    pub fn take_turn(&mut self, tile_index: usize) -> bool {
        let p = &mut self.players[self.current_player_idx];
        let tile = &p.tiles_in_hand[tile_index];
        let game_over = self.board.play_tile(p.board_index, tile);
        p.tiles_in_hand.remove(tile_index);
        if game_over {
            // TODO: remove this player from the active set,
            // and return their tiles to the stack / other players.
            return true;
        }
        if let Some(new_tile) = self.tile_stack.pop() {
            p.tiles_in_hand.push(new_tile);
        }
        self.current_player_idx += 1;
        self.current_player_idx %= self.players.len();
        false
    }
    pub fn rotate_tile(&mut self, player_idx: usize, tile_idx: usize) {
        let p = &mut self.players[player_idx];
        let tile = &mut p.tiles_in_hand[tile_idx];
        tile.rotate_left();
    }
    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player_idx]
    }
}
