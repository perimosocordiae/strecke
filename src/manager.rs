use crate::board::{Board, Position};
use crate::tiles::{all_tiles, Tile};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::cmp;

const TILES_PER_PLAYER: i32 = 3;

#[derive(Debug, Deserialize, Serialize)]
struct Player {
    username: String,
    board_index: usize,
    tiles_in_hand: Vec<Tile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlayerMove {
    pub tile_index: usize,
    pub row: usize,
    pub col: usize,
}

pub struct GameManager {
    board: Board,
    tile_stack: Vec<Tile>,
    players: Vec<Player>,
    current_player: usize,
}

impl GameManager {
    pub fn new(rng: &mut impl rand::Rng) -> Self {
        let mut tile_stack = all_tiles();
        tile_stack.shuffle(rng);
        GameManager {
            board: Board::new(),
            tile_stack,
            players: Vec::new(),
            current_player: 0,
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
    pub fn take_turn(&mut self, m: &PlayerMove) -> Result<bool, String> {
        let p = &mut self.players[self.current_player];
        let tile = &p.tiles_in_hand[m.tile_index];
        let game_over =
            self.board.play_tile(p.board_index, tile, m.row, m.col)?;
        p.tiles_in_hand.remove(m.tile_index);
        if let Some(new_tile) = self.tile_stack.pop() {
            p.tiles_in_hand.push(new_tile);
        }
        self.current_player += 1;
        Ok(game_over)
    }
    pub fn rotate_tile(&mut self, player_idx: usize, tile_idx: usize) {
        let p = &mut self.players[player_idx];
        let tile = &mut p.tiles_in_hand[tile_idx];
        tile.rotate_left();
    }
}
