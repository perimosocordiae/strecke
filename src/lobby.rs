use crate::board;
use crate::tiles::Port;
use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};

// No I,O
static CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";

pub fn generate_lobby_code() -> String {
    let range = Uniform::from(0..CODE_CHARS.len());
    let mut rng = rand::thread_rng();
    range
        .sample_iter(&mut rng)
        .take(4)
        .map(|x| char::from(CODE_CHARS[x]))
        .collect()
}

// Edge positions are ndexed in CW order starting from the top left (0,0,A).
// Valid range: [0, 48] (with 48=not ready).
type EdgePos = i8;
const NOT_READY: EdgePos = 48;

fn edge_position(pos: EdgePos) -> board::Position {
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
    board::Position {
        row,
        col,
        port,
        alive: true,
    }
}

#[derive(Serialize, Deserialize)]
pub struct Lobby {
    // Usernames of the present players
    names: Vec<String>,
    // Parallel vector of starting positions
    start_positions: Vec<EdgePos>,
    // Total number of players to allow
    max_num_players: usize,
}

impl Lobby {
    pub fn new(username: String) -> Self {
        Lobby {
            names: vec![username],
            start_positions: vec![NOT_READY],
            max_num_players: 2,
        }
    }

    fn host(&self) -> &String {
        &self.names[0]
    }

    pub fn player_names(&self) -> Vec<&String> {
        self.names
            .iter()
            .zip(self.start_positions.iter())
            .filter(|(_, pos)| *pos < &NOT_READY)
            .map(|(name, _)| name)
            .collect()
    }

    pub fn into_seated_players(
        self,
    ) -> impl std::iter::Iterator<Item = (String, board::Position)> {
        self.names
            .into_iter()
            .zip(self.start_positions.into_iter())
            .filter(|(_, pos)| *pos < NOT_READY)
            .map(|(name, pos)| (name, edge_position(pos)))
    }

    pub fn run_pregame_checks(&self, username: &str) -> Result<(), &str> {
        if self.max_num_players > 24 {
            return Err("Lobby is too large (max size is 24)");
        }
        if username != self.host() {
            return Err("Only the host can start the game");
        }
        if !self.start_positions.iter().any(|&x| x < NOT_READY) {
            return Err("No human players are ready to play");
        }
        Ok(())
    }

    pub fn prepare_for_game(&mut self) {
        let mut indices: Vec<usize> = (0..self.start_positions.len()).collect();
        indices.sort_by_key(|&i| &self.start_positions[i]);
        apply_permutation(indices.as_mut_slice(), self.names.as_mut_slice());
        apply_permutation(
            indices.as_mut_slice(),
            self.start_positions.as_mut_slice(),
        );
        let num_humans = 1 + self
            .start_positions
            .iter()
            .rposition(|&x| x < NOT_READY)
            .unwrap();
        assert!(num_humans <= self.max_num_players);
        if num_humans < self.max_num_players {
            let range = Uniform::from(0..48);
            let mut rng = rand::thread_rng();
            for (i, idx) in (num_humans..self.max_num_players).enumerate() {
                self.names[idx] = format!("AI player #{}", i + 1);
                // Assign a random starting location that isn't in use.
                // TODO: enforce separation constraints
                loop {
                    let pos = range.sample(&mut rng);
                    if !self.start_positions.contains(&pos) {
                        self.start_positions[idx] = pos;
                        break;
                    }
                }
            }
        }
    }
}

#[inline(always)]
fn toggle_mark_idx(idx: usize) -> usize {
    idx ^ isize::min_value() as usize
}

#[inline(always)]
fn idx_is_marked(idx: usize) -> bool {
    (idx & (isize::min_value() as usize)) != 0
}

fn apply_permutation<T>(indices: &mut [usize], slice: &mut [T]) {
    assert_eq!(slice.len(), indices.len());
    assert!(slice.len() <= isize::max_value() as usize);
    for i in 0..indices.len() {
        let i_idx = indices[i];
        if idx_is_marked(i_idx) {
            continue;
        }
        let mut j = i;
        let mut j_idx = i_idx;
        while j_idx != i {
            indices[j] = toggle_mark_idx(j_idx);
            slice.swap(j, j_idx);
            j = j_idx;
            j_idx = indices[j];
        }
        indices[j] = toggle_mark_idx(j_idx);
    }
    for idx in indices.iter_mut() {
        *idx = toggle_mark_idx(*idx);
    }
}

#[test]
fn test_make_code() {
    let code = generate_lobby_code();
    let escaped: String =
        code.chars().map(|c| c.escape_debug().to_string()).collect();
    assert_eq!(code.len(), 4);
    assert!(
        code.chars().all(|c| c.is_ascii_uppercase()),
        "code = '{}'",
        escaped
    );
}

#[test]
fn test_solo_lobby() {
    let x = Lobby::new("Bob".to_owned());
    assert_eq!(x.player_names(), Vec::<&String>::new());
}

#[test]
fn test_edge_position() {
    for pos in 0..48 {
        let board_pos = edge_position(pos);
        assert!(
            board_pos.is_valid_start(),
            "pos = {}, board_pos = {:?}",
            pos,
            board_pos
        );
    }
    assert_eq!(
        edge_position(0),
        board::Position {
            row: -1,
            col: 0,
            port: Port::F,
            alive: true
        }
    );
    assert_eq!(
        edge_position(1),
        board::Position {
            row: -1,
            col: 0,
            port: Port::E,
            alive: true
        }
    );
    assert_eq!(
        edge_position(2),
        board::Position {
            row: -1,
            col: 1,
            port: Port::F,
            alive: true
        }
    );
    assert_eq!(
        edge_position(24),
        board::Position {
            row: 6,
            col: 5,
            port: Port::B,
            alive: true
        }
    );
    assert_eq!(
        edge_position(35),
        board::Position {
            row: 6,
            col: 0,
            port: Port::A,
            alive: true
        }
    );
}
