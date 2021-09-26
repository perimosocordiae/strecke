use crate::board;
use crate::game::GameManager;
use argon2::{self, Config};
use chrono::Utc;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error;
use std::fmt;

#[derive(Deserialize)]
pub struct UserCredentials {
    username: String,
    #[serde(with = "serde_bytes")]
    password: Vec<u8>,
}

pub struct AppState {
    games: HashMap<i64, GameManager>,
    conn: rusqlite::Connection,
}

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug)]
struct LoginError;
impl fmt::Display for LoginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid login")
    }
}
impl error::Error for LoginError {}

#[derive(Debug)]
struct SignupError;
impl fmt::Display for SignupError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid username or password")
    }
}
impl error::Error for SignupError {}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = rusqlite::Connection::open(db_path)?;
        // Ensure DB tables exist.
        conn.execute(
            "CREATE TABLE IF NOT EXISTS players (
             id INTEGER PRIMARY KEY,
             username TEXT NOT NULL UNIQUE,
             hashed_password TEXT,
             num_games INTEGER,
             last_game TIMESTAMP
         )",
            [],
        )?;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS games (
             id INTEGER PRIMARY KEY,
             start_time TIMESTAMP,
             end_time TIMESTAMP,
             board_state JSON,
             player_ids JSON
         )",
            [],
        )?;
        // Remove any incomplete games.
        conn.execute("DELETE FROM games WHERE end_time IS NULL", [])?;
        Ok(Self {
            games: HashMap::new(),
            conn,
        })
    }

    pub fn new_game(
        &mut self,
        player_data: Vec<(String, board::Position)>,
    ) -> Result<i64> {
        let player_names: Vec<&String> =
            player_data.iter().map(|(name, _)| name).collect();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO games (start_time, player_ids) VALUES (?1, ?2)",
            [now.to_rfc3339(), serde_json::to_string(&player_names)?],
        )?;
        let game_id = self.conn.last_insert_rowid();
        let mut rng = rand::thread_rng();
        let mut gm = GameManager::new(&mut rng);
        for (username, position) in player_data.into_iter() {
            gm.register_player(username, position)?;
        }
        self.games.insert(game_id, gm);
        Ok(game_id)
    }

    pub fn game(&self, game_id: i64) -> &GameManager {
        &self.games[&game_id]
    }

    pub fn rotate_tile(
        &mut self,
        game_id: i64,
        username: &str,
        tile_idx: usize,
    ) -> Result<&str> {
        let game = self.games.get_mut(&game_id).ok_or("Invalid game ID")?;
        let player = game.mut_player(&username).ok_or("No such player")?;
        player.rotate_tile(tile_idx);
        Ok("OK")
    }

    pub fn take_turn(
        &mut self,
        game_id: i64,
        username: &str,
        idx: usize,
    ) -> Result<&str> {
        let game = self.games.get_mut(&game_id).ok_or("Invalid game ID")?;
        if game.current_player().username != username {
            return Ok("not your turn");
        }
        let num_alive = game.take_turn(idx);
        if num_alive >= 2 {
            return Ok("OK");
        }
        // Game is over, record the result in the DB.
        let now = Utc::now();
        self.conn.execute(
            "UPDATE games
            SET board_state = ?1, end_time = ?2
            WHERE id = ?3 LIMIT 1",
            [
                serde_json::to_string(&game.board)?,
                now.to_rfc3339(),
                game_id.to_string(),
            ],
        )?;
        let players_json = self.conn.query_row(
            "SELECT player_ids FROM games WHERE id = ?1 LIMIT 1",
            [game_id.to_string()],
            |row| row.get::<usize, String>(0),
        )?;
        let player_names: Vec<String> = serde_json::from_str(&players_json)?;
        for name in player_names.into_iter() {
            self.conn.execute(
                "UPDATE players
                SET num_games = num_games + 1, last_game = ?1
                WHERE username = ?2 LIMIT 1",
                [now.to_rfc3339(), name],
            )?;
        }
        // TODO: signal that the game is over in a better way
        if num_alive == 1 {
            // We have a winner, named game.current_player().username
            return Ok("Somebody won!");
        }
        Ok("Everyone loses :(")
    }

    pub fn sign_up(
        &mut self,
        creds: UserCredentials,
        secret: &jsonwebtoken::EncodingKey,
    ) -> Result<String> {
        let salt = rand::thread_rng().gen::<[u8; 32]>();
        let config = Config::default();
        let hash = argon2::hash_encoded(&creds.password, &salt, &config)?;
        match self.conn.execute(
            "INSERT INTO players (username, hashed_password) VALUES (?, ?)",
            [&creds.username, &hash],
        ) {
            Ok(_) => {
                info!("New user signed up: {}", &creds.username);
                Ok(create_jwt(&creds.username, secret)?)
            }
            Err(e) => {
                error!("Signup error: {:?}", e);
                Err(SignupError.into())
            }
        }
    }

    pub fn check_login(
        &self,
        creds: UserCredentials,
        secret: &jsonwebtoken::EncodingKey,
    ) -> Result<String> {
        match self.conn.query_row(
            "SELECT hashed_password FROM players WHERE username = ?1",
            [&creds.username],
            |row| row.get::<usize, String>(0),
        ) {
            Ok(db_pw) => {
                if argon2::verify_encoded(&db_pw, &creds.password)? {
                    info!("User logged in: {}", &creds.username);
                    Ok(create_jwt(&creds.username, secret)?)
                } else {
                    Err(LoginError.into())
                }
            }
            Err(e) => {
                match e {
                    // Typical case when username doesn't exist.
                    rusqlite::Error::QueryReturnedNoRows => {}
                    // For any other error, log it.
                    _ => {
                        error!("Login query failure: {:?}", e);
                    }
                };
                Err(LoginError.into())
            }
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    exp: usize,
}

fn create_jwt(
    username: &str,
    secret: &jsonwebtoken::EncodingKey,
) -> Result<String> {
    let expiration = Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp();
    let claims = Claims {
        sub: username.to_owned(),
        exp: expiration as usize,
    };
    let header = jsonwebtoken::Header::new(jsonwebtoken::Algorithm::HS512);
    jsonwebtoken::encode(&header, &claims, secret).map_err(|e| e.into())
}

pub fn decode_jwt(
    token: &str,
    key: &jsonwebtoken::DecodingKey,
) -> Result<String> {
    let decoded = jsonwebtoken::decode::<Claims>(
        token,
        key,
        &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS512),
    )?;
    Ok(decoded.claims.sub)
}
