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
    pub tiles_in_hand: Vec<Tile>,
}

pub struct GameManager {
    pub board: Board,
    tile_stack: Vec<Tile>,
    pub alive_players: Vec<Player>,
    current_player_idx: usize,
}

impl GameManager {
    pub fn new(rng: &mut impl rand::Rng) -> Self {
        let mut tile_stack = all_tiles();
        tile_stack.shuffle(rng);
        GameManager {
            board: Board::default(),
            tile_stack,
            alive_players: Vec::new(),
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
        self.alive_players.push(Player {
            username,
            board_index: self.board.add_player(start_position)?,
            tiles_in_hand: self.tile_stack.split_off(pos),
        });
        Ok(())
    }
    pub fn take_turn(&mut self, tile_index: usize, facing: Direction) -> usize {
        let bidx = self.alive_players[self.current_player_idx].board_index;
        {
            let p = &mut self.alive_players[self.current_player_idx];
            if tile_index < p.tiles_in_hand.len() {
                let tile = p.tiles_in_hand.remove(tile_index);
                self.board.play_tile(bidx, &tile, facing);
            }
        }
        // Check for any newly-dead players.
        let mut newly_dead = false;
        let mut idx = 0;
        while idx < self.alive_players.len() {
            let bidx = self.alive_players[idx].board_index;
            if !self.board.players[bidx].last().unwrap().alive {
                newly_dead = true;
                // Remove this player from the active list.
                let mut dead = self.alive_players.remove(idx);
                // Return tiles to the stack.
                self.tile_stack.append(&mut dead.tiles_in_hand);
            } else {
                idx += 1;
            }
        }
        // Allocate tiles from the tile stack.
        // TODO: use the actual game logic here, this is a hack.
        for p in self.alive_players.iter_mut() {
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
                self.alive_players.iter().position(|p| p.board_index > bidx)
            {
                self.current_player_idx = idx;
            } else {
                self.current_player_idx = 0;
            }
        } else {
            self.current_player_idx += 1;
            self.current_player_idx %= self.alive_players.len();
        }
        self.alive_players.len()
    }
    pub fn current_player(&self) -> &Player {
        &self.alive_players[self.current_player_idx]
    }
    pub fn current_player_pos(&self) -> &Position {
        self.board.players[self.current_player().board_index]
            .last()
            .unwrap()
    }
    pub fn get_player(&self, player_name: &str) -> Option<&Player> {
        self.alive_players
            .iter()
            .find(|&p| p.username == player_name)
    }
    pub fn player_trail_lengths(&self) -> Vec<i32> {
        self.board.players.iter().map(|p| p.len() as i32).collect()
    }
    pub fn is_alive(&self, player: &Player) -> bool {
        self.board
            .players
            .get(player.board_index)
            .map(|p| p.last().unwrap().alive)
            .unwrap_or(false)
    }
    pub fn is_over(&self) -> bool {
        self.alive_players.len() <= 1
    }
}
