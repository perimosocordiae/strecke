use crate::tiles::{Direction, Port, Tile};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, PartialEq, Clone)]
pub struct Position {
    pub row: i8,
    pub col: i8,
    pub port: Port,
    pub alive: bool,
}

impl Position {
    pub fn is_valid_start(&self) -> bool {
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
    fn next_tile_coords(&self) -> (i8, i8) {
        match self.port.facing_side() {
            Direction::North => (self.row - 1, self.col),
            Direction::South => (self.row + 1, self.col),
            Direction::East => (self.row, self.col + 1),
            Direction::West => (self.row, self.col - 1),
        }
    }
    pub fn next_tile_position(&self) -> Self {
        let (row, col) = self.next_tile_coords();
        Self {
            row,
            col,
            port: self.port.flip(),
            alive: self.alive,
        }
    }
    pub fn l1_distance(&self, other: &Position) -> i32 {
        (self.row - other.row).abs() as i32
            + (self.col - other.col).abs() as i32
    }
}

// Edge positions are ndexed in CW order starting from the top left (0,0,A).
// Valid range: [0, 48] (with 48=not ready).
pub type EdgePos = i8;
pub const NOT_READY: EdgePos = 48;

pub fn is_valid_edge_position(pos: EdgePos) -> bool {
    (0..=NOT_READY).contains(&pos)
}

pub fn edge_position(pos: EdgePos) -> Position {
    let port: Port;
    let (row, col) = if pos < 12 {
        port = if pos % 2 == 0 { Port::F } else { Port::E };
        (-1, pos / 2)
    } else if pos < 24 {
        port = if pos % 2 == 0 { Port::H } else { Port::G };
        ((pos - 12) / 2, 6)
    } else if pos < 36 {
        port = if pos % 2 == 0 { Port::B } else { Port::A };
        (6, (35 - pos) / 2)
    } else if pos < 48 {
        port = if pos % 2 == 0 { Port::D } else { Port::C };
        ((47 - pos) / 2, -1)
    } else {
        panic!("Invalid EdgePos: {}", pos);
    };
    Position {
        row,
        col,
        port,
        alive: true,
    }
}

#[test]
fn test_is_valid_start() {
    assert!(
        Position {
            row: -1,
            col: 2,
            port: Port::E,
            alive: true
        }
        .is_valid_start()
    );
}

#[derive(Debug, PartialEq, Clone)]
pub enum StepResult {
    Moved(Position),
    OffBoard(Position),
    Blocked(Position),
}

#[derive(Default, Debug, Deserialize, Serialize, Clone)]
pub struct Board {
    // 2d array of tiles and their orientations
    grid: [[Option<(Tile, Direction)>; 6]; 6],
    // each player has a trail of positions, most recent at the end
    pub players: Vec<Vec<Position>>,
}

impl Board {
    pub fn get_tile(
        &self,
        pos: &Position,
    ) -> Option<&Option<(Tile, Direction)>> {
        if !(0..6).contains(&pos.row) || !(0..6).contains(&pos.col) {
            return None;
        }
        Some(&self.grid[pos.row as usize][pos.col as usize])
    }
    pub fn add_player(&mut self, pos: Position) -> Result<usize, String> {
        if !pos.is_valid_start() {
            return Err(format!("Invalid starting position: {:?}", pos));
        }
        self.players.push(vec![pos]);
        Ok(self.players.len() - 1)
    }
    pub fn step(
        &self,
        pos: &Position,
        virtual_tile: Option<((i8, i8), Tile, Direction)>,
    ) -> StepResult {
        let mut next_pos = pos.next_tile_position();
        let row = next_pos.row;
        let col = next_pos.col;

        if !(0..6).contains(&row) || !(0..6).contains(&col) {
            next_pos.alive = false;
            return StepResult::OffBoard(next_pos);
        }

        let tile_opt = if let Some(((vr, vc), t, d)) = virtual_tile {
            if row == vr && col == vc {
                Some((t, d))
            } else {
                self.grid[row as usize][col as usize]
            }
        } else {
            self.grid[row as usize][col as usize]
        };

        match tile_opt {
            Some((tile, facing)) => {
                next_pos.port = tile.traverse(next_pos.port, facing);
                StepResult::Moved(next_pos)
            }
            None => StepResult::Blocked(next_pos),
        }
    }

    pub fn play_tile(
        &mut self,
        player_idx: usize,
        tile: &Tile,
        facing: Direction,
    ) {
        // Add the new tile in the target location.
        if let Some(pos) = self.players[player_idx].last() {
            let (row, col) = pos.next_tile_coords();
            self.grid[row as usize][col as usize] = Some((*tile, facing));
        }
        // Move all players, if still alive.
        for i in 0..self.players.len() {
            while let Some(pos) = self.players[i].last() {
                if !pos.alive {
                    break;
                }
                // We need to clone pos to avoid borrowing self.players immutably while self is needed for step
                let pos = pos.clone();
                match self.step(&pos, None) {
                    StepResult::Moved(new_pos) => self.players[i].push(new_pos),
                    StepResult::OffBoard(dead_pos) => {
                        self.players[i].push(dead_pos);
                        break;
                    }
                    StepResult::Blocked(_) => break,
                }
            }
        }
    }
}

#[test]
fn test_default_board() {
    let b = Board::default();
    assert!(b.grid[0][0].is_none());
}

#[test]
fn test_add_players() {
    let mut b = Board::default();
    assert_eq!(b.players.len(), 0);
    assert_eq!(
        b.add_player(Position {
            row: 1,
            col: -1,
            port: Port::D,
            alive: true,
        }),
        Ok(0)
    );
    assert_eq!(b.players.len(), 1);
    assert_eq!(b.players[0].len(), 1);
    assert_eq!(b.players[0][0].port, Port::D);
}
