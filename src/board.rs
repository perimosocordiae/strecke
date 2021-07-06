use crate::tiles::{Direction, Port, Tile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Position {
    pub row: i8,
    pub col: i8,
    pub port: Port,
    pub alive: bool,
}

impl Position {
    fn is_valid_start(&self) -> bool {
        if self.col < -1 || self.col > 6 || !self.alive {
            return false;
        }
        match self.port {
            Port::A | Port::B => self.row == 6,
            Port::C | Port::D => self.col == -1,
            Port::E | Port::F => self.row == -1,
            Port::G | Port::H => self.col == 6,
        }
    }
    fn next_tile_position(&self) -> (i8, i8) {
        match self.port.facing_side() {
            Direction::North => (self.row - 1, self.col),
            Direction::South => (self.row + 1, self.col),
            Direction::East => (self.row, self.col + 1),
            Direction::West => (self.row, self.col - 1),
        }
    }
    fn update(&mut self, r: i8, c: i8, tile: &Tile) {
        self.row = r;
        self.col = c;
        self.port = tile.traverse(self.port);
    }
}

#[test]
fn test_is_valid_start() {
    assert!(Position {
        row: -1,
        col: 2,
        port: Port::E,
        alive: true
    }
    .is_valid_start());
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Board {
    grid: [[Option<Tile>; 6]; 6],
    pub players: Vec<Position>,
}

impl Board {
    pub fn new() -> Self {
        Board {
            grid: [[None; 6]; 6],
            players: vec![],
        }
    }
    pub fn add_player(&mut self, pos: Position) -> Result<usize, String> {
        if !pos.is_valid_start() {
            return Err(format!("Invalid starting position: {:?}", pos));
        }
        self.players.push(pos);
        Ok(self.players.len() - 1)
    }
    pub fn play_tile(&mut self, player_idx: usize, tile: &Tile) {
        let pos = &mut self.players[player_idx];
        let (row, col) = pos.next_tile_position();
        pos.update(row, col, &tile);
        self.grid[row as usize][col as usize] = Some(*tile);
        self.move_players();
    }
    // TODO: record the trajectory of each player?
    fn move_players(&mut self) {
        for pos in self.players.iter_mut() {
            while pos.alive {
                let (d_row, d_col) = pos.port.facing_side().grid_offsets();
                let r = pos.row + d_row;
                let c = pos.col + d_col;
                if !(0..6).contains(&r) || !(0..6).contains(&c) {
                    pos.row = r;
                    pos.col = c;
                    pos.port = pos.port.flip();
                    pos.alive = false;
                } else {
                    match self.grid[r as usize][c as usize] {
                        // Hit another tile, traverse and keep looping.
                        Some(t) => pos.update(r, c, &t),
                        // Hit a blank cell, stop iterating.
                        None => break,
                    }
                }
            }
        }
    }
}

#[test]
fn test_new_board() {
    let b = Board::new();
    assert!(b.grid[0][0].is_none());
}

#[test]
fn test_add_players() {
    let mut b = Board::new();
    assert_eq!(b.players.len(), 0);
    assert_eq!(
        b.add_player(Position {
            row: 1,
            col: -1,
            port: Port::D,
            alive: true,
        })
        .unwrap(),
        0
    );
    assert_eq!(b.players.len(), 1);
    assert_eq!(b.players[0].port, Port::D);
}
