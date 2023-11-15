use crate::board::{Board, Position};
use crate::tiles::{all_tiles, Direction, Tile};
use log::info;
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
    pub current_player_idx: usize,
    dragon_player_bidx: Option<usize>,
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
            dragon_player_bidx: None,
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
    pub fn take_turn(
        &mut self,
        tile_index: usize,
        facing: Direction,
    ) -> Option<Vec<String>> {
        let bidx = self.alive_players[self.current_player_idx].board_index;
        {
            let p = &mut self.alive_players[self.current_player_idx];
            if tile_index < p.tiles_in_hand.len() {
                let tile = p.tiles_in_hand.remove(tile_index);
                self.board.play_tile(bidx, &tile, facing);
                // Replace the played tile, if possible.
                if let Some(new_tile) = self.tile_stack.pop() {
                    p.tiles_in_hand.push(new_tile);
                } else if self.dragon_player_bidx.is_none() {
                    self.dragon_player_bidx = Some(bidx);
                }
            }
        }
        // Check for any newly-dead players.
        if self.remove_dead_players() {
            // Check for game over.
            if self.alive_players.len() <= 1 {
                return Some(
                    self.alive_players
                        .iter()
                        .map(|p| p.username.clone())
                        .collect(),
                );
            }
            // Distribute tiles starting from the dragon player.
            self.distribute_tiles();
            // Update the current player index.
            if let Some(idx) =
                self.alive_players.iter().position(|p| p.board_index > bidx)
            {
                self.current_player_idx = idx;
            } else {
                self.current_player_idx = 0;
            }
        } else {
            // Move to the next alive player.
            self.current_player_idx += 1;
            self.current_player_idx %= self.alive_players.len();
        }
        if self.is_over() {
            // All remaining players win!
            Some(
                self.alive_players
                    .iter()
                    .map(|p| p.username.clone())
                    .collect(),
            )
        } else {
            // Game is still going.
            None
        }
    }
    fn remove_dead_players(&mut self) -> bool {
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
                // If they were the dragon player, give the dragon to the next
                // player missing tiles.
                if self.dragon_player_bidx == Some(bidx) {
                    // Technically this should go in order starting from the
                    // current player, but this is easier for now.
                    self.dragon_player_bidx = self
                        .alive_players
                        .iter()
                        .filter(|p| {
                            p.tiles_in_hand.len() < TILES_PER_PLAYER as usize
                        })
                        .next()
                        .map(|p| p.board_index);
                }
                info!("Player died: {}", dead.username);
            } else {
                idx += 1;
            }
        }
        newly_dead
    }
    fn distribute_tiles(&mut self) {
        if self.tile_stack.is_empty() || self.dragon_player_bidx.is_none() {
            return;
        }
        let tile_limit = TILES_PER_PLAYER as usize;
        // We know the dragon player is alive, so unwrap is safe.
        let dragon_idx = self
            .alive_players
            .iter()
            .position(|p| p.board_index == self.dragon_player_bidx.unwrap())
            .unwrap();
        // Feed the dragon and reset.
        self.alive_players[dragon_idx]
            .tiles_in_hand
            .push(self.tile_stack.pop().unwrap());
        self.dragon_player_bidx = None;
        // Feed any other players who need tiles.
        let mut num_loops = 0;
        let mut idx = dragon_idx;
        while num_loops < TILES_PER_PLAYER {
            idx += 1;
            idx %= self.alive_players.len();
            // Hacky way to break out of an endless loop.
            if idx == dragon_idx {
                num_loops += 1;
            }
            // Skip players with a full hand.
            if self.alive_players[idx].tiles_in_hand.len() >= tile_limit {
                continue;
            }
            // Give them a tile, if possible, otherwise make them the dragon.
            if let Some(tile) = self.tile_stack.pop() {
                self.alive_players[idx].tiles_in_hand.push(tile);
            } else {
                self.dragon_player_bidx =
                    Some(self.alive_players[idx].board_index);
                break;
            }
        }
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
    pub fn player_scores(&self) -> Vec<i32> {
        self.board
            .players
            .iter()
            .map(|trail| {
                let n = trail.len() as i32 - 1;
                if trail.last().unwrap().alive {
                    n + 1000
                } else {
                    n - 1
                }
            })
            .collect()
    }
    pub fn is_over(&self) -> bool {
        self.alive_players.len() <= 1
            || (self.tile_stack.is_empty()
                && self.current_player().tiles_in_hand.is_empty())
    }
}
