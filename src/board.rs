use crate::tiles::{Direction, Port, Tile};

#[derive(Debug)]
pub struct Position {
    pub row: i8,
    pub col: i8,
    pub port: Port,
}

impl Position {
    fn is_valid_start(&self) -> bool {
        if self.col < -1 || self.col > 6 {
            return false;
        }
        match self.port {
            Port::A | Port::B => self.row == 6,
            Port::C | Port::D => self.col == -1,
            Port::E | Port::F => self.row == -1,
            Port::G | Port::H => self.col == 6,
        }
    }
    fn is_valid_direction(&self, dir: Direction) -> bool {
        self.port.facing_side() == dir
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
        port: Port::E
    }
    .is_valid_start());
}

#[test]
fn test_is_valid_direction() {
    assert!(Position {
        row: 5,
        col: 2,
        port: Port::C
    }
    .is_valid_direction(Direction::East));
}

#[derive(Debug)]
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
    pub fn play_tile(
        &mut self,
        player_idx: usize,
        tile: Tile,
        row: usize,
        col: usize,
    ) -> Result<bool, String> {
        let pos = &mut self.players[player_idx];
        if row >= 6 || col >= 6 {
            return Err(format!("Invalid tile location: ({}, {})", row, col));
        }
        let side = adjacent_side(pos.row, pos.col, row as i8, col as i8)?;
        if !pos.is_valid_direction(side) {
            return Err(format!(
                "Invalid tile placement: ({}, {}) does not connect to player's route",
                row, col
            ));
        }
        pos.update(row as i8, col as i8, &tile);
        self.grid[row][col] = Some(tile);
        // TODO: return a sequence of positions
        loop {
            let (d_row, d_col) = pos.port.facing_side().grid_offsets();
            let r = pos.row + d_row;
            let c = pos.col + d_col;
            if !(0..6).contains(&r) || !(0..6).contains(&c) {
                pos.row = r;
                pos.col = c;
                pos.port = pos.port.flip();
                return Ok(true); // Game over for this player.
            }
            match self.grid[r as usize][c as usize] {
                // Hit another tile, traverse and keep looping.
                Some(t) => pos.update(r, c, &t),
                // Hit a blank cell, stop.
                None => return Ok(false),
            }
        }
    }
}

fn adjacent_side(src_row: i8, src_col: i8, dst_row: i8, dst_col: i8) -> Result<Direction, String> {
    match (dst_row - src_row, dst_col - src_col) {
        (0, -1) => Ok(Direction::West),
        (0, 1) => Ok(Direction::East),
        (-1, 0) => Ok(Direction::North),
        (1, 0) => Ok(Direction::South),
        _ => Err(format!(
            "Invalid tile placement: ({}, {}) is not adjacent to ({}, {})",
            dst_row, dst_col, src_row, src_col
        )),
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
        })
        .unwrap(),
        0
    );
    assert_eq!(b.players.len(), 1);
    assert_eq!(b.players[0].port, Port::D);
}
