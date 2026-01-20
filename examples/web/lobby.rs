use rand::distr::{Distribution, Uniform};
use rand::prelude::IndexedRandom;
use serde::{Deserialize, Serialize};
use strecke::board;
use strecke::board::edge_position;
use strecke::tiles::Port;

const MAX_PLAYERS: usize = 11;
// No I,O
static CODE_CHARS: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";

pub fn generate_lobby_code() -> String {
    let range = Uniform::try_from(0..CODE_CHARS.len()).unwrap();
    let mut rng = rand::rng();
    range
        .sample_iter(&mut rng)
        .take(4)
        .map(|x| char::from(CODE_CHARS[x]))
        .collect()
}

#[derive(Serialize, Deserialize)]
pub struct Lobby {
    // Usernames of the present players
    names: Vec<String>,
    // Parallel vector of starting positions
    start_positions: Vec<board::EdgePos>,
    // Total number of players to allow
    max_num_players: usize,
}

impl Lobby {
    pub fn new(username: String) -> Self {
        let max_num_players = 2;
        let mut names = Vec::with_capacity(max_num_players);
        names.push(username);
        let mut start_positions = Vec::with_capacity(max_num_players);
        start_positions.push(board::NOT_READY);
        Lobby {
            names,
            start_positions,
            max_num_players,
        }
    }

    pub fn host(&self) -> &String {
        &self.names[0]
    }

    pub fn resize(&mut self, new_size: usize) -> Result<(), &str> {
        if new_size > MAX_PLAYERS {
            return Err("Too many players");
        }
        self.max_num_players = new_size;
        if self.names.len() > new_size {
            self.names.truncate(new_size);
            self.start_positions.truncate(new_size);
        }
        Ok(())
    }

    pub fn take_seat(
        &mut self,
        seat_idx: board::EdgePos,
        username: String,
    ) -> Result<(), &str> {
        if !board::is_valid_edge_position(seat_idx) {
            return Err("Invalid seat_idx");
        }
        if let Some(i) = self.names.iter().position(|name| name == &username) {
            self.start_positions[i] = seat_idx;
        } else {
            self.names.push(username);
            self.start_positions.push(seat_idx);
        }
        Ok(())
    }

    pub fn player_names(&self) -> Vec<&String> {
        self.names
            .iter()
            .zip(self.start_positions.iter())
            .filter(|(_, pos)| *pos < &board::NOT_READY)
            .map(|(name, _)| name)
            .collect()
    }

    pub fn into_seated_players(
        self,
    ) -> impl std::iter::Iterator<Item = (String, board::Position)> {
        self.names
            .into_iter()
            .zip(self.start_positions.into_iter())
            .filter(|(_, pos)| *pos < board::NOT_READY)
            .map(|(name, pos)| (name, board::edge_position(pos)))
    }

    pub fn run_pregame_checks(&self, username: &str) -> Result<(), &str> {
        if self.max_num_players > MAX_PLAYERS {
            return Err("Lobby has too many players");
        }
        if username != self.host() {
            return Err("Only the host can start the game");
        }
        if !self.start_positions.iter().any(|&x| x < board::NOT_READY) {
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
            .rposition(|&x| x < board::NOT_READY)
            .unwrap();
        assert!(num_humans <= self.max_num_players);
        if num_humans < self.max_num_players {
            self.names.truncate(num_humans);
            self.start_positions.truncate(num_humans);
            let mut rng = rand::rng();
            for i in 0..(self.max_num_players - num_humans) {
                self.names.push(format!("AI player #{}", i + 1));
                // Assign a starting location with max separation from other players.
                let candidates: Vec<board::EdgePos> = (0..48)
                    .map(|x| x as board::EdgePos)
                    .filter(|x| !self.start_positions.contains(x))
                    .collect();

                if candidates.is_empty() {
                    break;
                }

                // Helper to get entry tile coords
                let get_coords = |pos: board::EdgePos| {
                    let p = board::edge_position(pos);
                    let next = p.next_tile_position();
                    (next.row, next.col)
                };

                let best_candidates = if self.start_positions.is_empty() {
                    candidates
                } else {
                    let mut max_min_dist = -1;
                    let mut bests = Vec::new();

                    for &cand in &candidates {
                        let (r, c) = get_coords(cand);
                        let mut min_dist = i32::MAX;
                        for &existing in &self.start_positions {
                            let (er, ec) = get_coords(existing);
                            let dist = (r - er).abs() as i32 + (c - ec).abs() as i32;
                            if dist < min_dist {
                                min_dist = dist;
                            }
                        }

                        if min_dist > max_min_dist {
                            max_min_dist = min_dist;
                            bests = vec![cand];
                        } else if min_dist == max_min_dist {
                            bests.push(cand);
                        }
                    }
                    bests
                };

                if let Some(&chosen) = best_candidates.choose(&mut rng) {
                    self.start_positions.push(chosen);
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

#[test]
fn test_ai_separation() {
    let mut lobby = Lobby::new("Alice".to_string());
    // Host takes seat 0 (Top-Left, entering (0,0))
    lobby.take_seat(0, "Alice".to_string()).unwrap();

    // Resize to 2 players (1 human, 1 AI)
    lobby.resize(2).unwrap();

    lobby.prepare_for_game();

    // Verify positions
    let positions = &lobby.start_positions;
    assert_eq!(positions.len(), 2);

    let p0 = board::edge_position(positions[0]);
    let p1 = board::edge_position(positions[1]);

    let p0_next = p0.next_tile_position();
    let p1_next = p1.next_tile_position();

    let dist = (p0_next.row - p1_next.row).abs() + (p0_next.col - p1_next.col).abs();

    // With 2 players, they should be very far apart.
    assert!(dist >= 5, "AI spawned too close! dist={}", dist);
}

#[test]
fn test_ai_separation_many() {
    let mut lobby = Lobby::new("Alice".to_string());
    lobby.take_seat(0, "Alice".to_string()).unwrap();
    lobby.resize(11).unwrap(); // Max players
    lobby.prepare_for_game();

    let positions = &lobby.start_positions;
    assert_eq!(positions.len(), 11);

    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
             let p1 = board::edge_position(positions[i]).next_tile_position();
             let p2 = board::edge_position(positions[j]).next_tile_position();
             let dist = (p1.row - p2.row).abs() + (p1.col - p2.col).abs();
             assert!(dist > 0, "Players {} and {} share entry tile!", i, j);
        }
    }
}
