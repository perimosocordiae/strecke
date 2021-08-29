use crate::game::GameManager;
use crate::{board, tiles};
use chrono::Utc;
use rusqlite::{Connection, Result};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize)]
pub struct UserCredentials {
    username: String,
    password: String,
}

pub struct AppState {
    games: HashMap<i64, GameManager>,
    conn: Connection,
}

impl AppState {
    pub fn new(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)?;
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
        )
        .unwrap();
        gm.register_player(
            "Bob",
            board::Position {
                row: -1,
                col: 3,
                port: tiles::Port::E,
                alive: true,
            },
        )
        .unwrap();
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

    pub fn sign_up(&mut self, creds: UserCredentials) -> Result<()> {
        self.conn.execute(
            "INSERT INTO players (username, hashed_password) VALUES (?, ?)",
            [&creds.username, &creds.password],
        )?;
        Ok(())
    }

    pub fn check_login(&self, creds: UserCredentials) -> Result<bool> {
        let db_pw: String = self.conn.query_row(
            "SELECT hashed_password FROM players WHERE username = ?1",
            [&creds.username],
            |row| row.get(0),
        )?;
        Ok(db_pw == creds.password)
    }
}
