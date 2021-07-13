use crate::board::{Board, Position};
use crate::tiles::{all_tiles, Tile};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::cmp;

// How large each player's "hand" can be.
const TILES_PER_PLAYER: i32 = 3;

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    username: String,
    board_index: usize,
    tiles_in_hand: Vec<Tile>,
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
    pub fn take_turn(&mut self, tile_index: usize) -> usize {
        let bidx = self.players[self.current_player_idx].board_index;
        {
            let p = &mut self.players[self.current_player_idx];
            self.board.play_tile(bidx, &p.tiles_in_hand[tile_index]);
            p.tiles_in_hand.remove(tile_index);
        }
        let dead_players: Vec<usize> = self
            .board
            .players
            .iter()
            .enumerate()
            .filter(|(_, pos)| !pos.alive)
            .map(|(idx, _)| idx)
            .collect();
        // Check for any newly-dead players.
        let mut newly_dead = false;
        let mut idx = 0;
        while idx < self.players.len() {
            if dead_players.contains(&self.players[idx].board_index) {
                newly_dead = true;
                // Remove this player from the active list.
                let mut dead = self.players.remove(idx);
                // Return tiles to the stack.
                self.tile_stack.append(&mut dead.tiles_in_hand);
            } else {
                idx += 1;
            }
        }
        // Allocate tiles from the tile stack.
        // TODO: use the actual game logic here, this is a hack.
        for p in self.players.iter_mut() {
            if p.tiles_in_hand.len() < 3 {
                if let Some(new_tile) = self.tile_stack.pop() {
                    p.tiles_in_hand.push(new_tile);
                } else {
                    break;
                }
            }
        }
        // Update the current player index.
        if newly_dead {
            if let Some(idx) =
                self.players.iter().position(|p| p.board_index > bidx)
            {
                self.current_player_idx = idx;
            } else {
                self.current_player_idx = 0;
            }
        } else {
            self.current_player_idx += 1;
            self.current_player_idx %= self.players.len();
        }
        self.players.len()
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
