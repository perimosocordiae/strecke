use rand::distributions::{Distribution, Uniform};
use serde::{Deserialize, Serialize};

// No I,O
static CODE_CHARS: &'static [u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ";

pub fn generate_lobby_code() -> String {
    let range = Uniform::from(0..CODE_CHARS.len());
    let mut rng = rand::thread_rng();
    range
        .sample_iter(&mut rng)
        .take(4)
        .map(|x| char::from(x as u8))
        .collect()
}

#[test]
fn test_make_code() {
    let code = generate_lobby_code();
    assert_eq!(code.len(), 4);
}

#[derive(Serialize, Deserialize)]
pub struct Lobby {
    // Usernames of the present players
    names: Vec<String>,
    // Indexed in CW order starting from the top left (0,0,A).
    // Valid range: [0, 48] (with 48=not selected).
    start_positions: Vec<usize>,
}
