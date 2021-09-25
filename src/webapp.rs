use crate::game::GameManager;
use crate::{board, tiles};
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

    pub fn new_game(&mut self) -> Result<i64> {
        let mut rng = rand::thread_rng();
        let mut gm = GameManager::new(&mut rng);
        // Start demo code to exercise the game logic--------------------
        gm.register_player(
            "CJ",
            board::Position {
                row: 2,
                col: 6,
                port: tiles::Port::G,
                alive: true,
            },
        )?;
        gm.register_player(
            "Bob",
            board::Position {
                row: -1,
                col: 3,
                port: tiles::Port::E,
                alive: true,
            },
        )?;
        // End demo code ------------------------------------------------
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO games (start_time) VALUES (?1)",
            [now.to_rfc3339()],
        )?;
        let game_id = self.conn.last_insert_rowid();
        self.games.insert(game_id, gm);
        Ok(game_id)
    }

    pub fn game(&self, game_id: i64) -> &GameManager {
        &self.games[&game_id]
    }

    pub fn mut_game(&mut self, game_id: i64) -> &mut GameManager {
        self.games.get_mut(&game_id).unwrap()
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
