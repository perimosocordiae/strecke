use crate::agent;
use crate::board::{Board, Position};
use crate::tiles::{all_tiles, Direction, Tile};
use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};
use std::cmp;

// How large each player's "hand" can be.
const TILES_PER_PLAYER: i32 = 3;

#[derive(Debug, Deserialize, Serialize)]
pub struct Player {
    pub username: String,
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
            board: Board::default(),
            tile_stack,
            players: Vec::new(),
            current_player_idx: 0,
        }
    }
    pub fn register_player(
        &mut self,
        username: String,
        start_position: Position,
    ) -> Result<(), String> {
        let pos = cmp::max(0, self.tile_stack.len() as i32 - TILES_PER_PLAYER)
            as usize;
        self.players.push(Player {
            username,
            board_index: self.board.add_player(start_position)?,
            tiles_in_hand: self.tile_stack.split_off(pos),
        });
        Ok(())
    }
    pub fn take_turn(&mut self, tile_index: usize, facing: Direction) -> usize {
        let bidx = self.players[self.current_player_idx].board_index;
        {
            let p = &mut self.players[self.current_player_idx];
            if tile_index < p.tiles_in_hand.len() {
                let tile = p.tiles_in_hand.remove(tile_index);
                self.board.play_tile(bidx, &tile, facing);
            }
        }
        // Check for any newly-dead players.
        let mut newly_dead = false;
        let mut idx = 0;
        while idx < self.players.len() {
            let bidx = self.players[idx].board_index;
            if !self.board.players[bidx].last().unwrap().alive {
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
        // HACK: Handle AI player moves.
        if self.players.len() > 1
            && self.current_player().username.starts_with("AI player #")
        {
            let mut bidxs: Vec<usize> =
                self.players.iter().map(|p| p.board_index).collect();
            bidxs.swap(self.current_player_idx, 0);
            let ai_move = agent::select_tile(
                &self.board,
                &bidxs,
                &self.current_player().tiles_in_hand,
            );
            return self.take_turn(ai_move.0, ai_move.1);
        }
        self.players.len()
    }
    pub fn current_player(&self) -> &Player {
        &self.players[self.current_player_idx]
    }
    pub fn get_player(&self, player_name: &str) -> Option<&Player> {
        self.players.iter().find(|&p| p.username == player_name)
    }
}
