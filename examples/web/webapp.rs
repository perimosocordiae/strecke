use argon2::{self, Config};
use chrono::Utc;
use log::{error, info};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::{hash_map::Entry, HashMap};
use std::error;
use std::fmt;
use strecke::board;
use strecke::game::GameManager;
use strecke::lobby;
use strecke::tiles::Direction;

#[derive(Deserialize)]
pub struct UserCredentials {
    username: String,
    #[serde(with = "serde_bytes")]
    password: Vec<u8>,
}

#[derive(Deserialize)]
pub struct TurnParams {
    game_id: i64,
    idx: usize,
    facing: Direction,
}

#[derive(Serialize)]
#[serde(tag = "action")]
pub enum LobbyResponse<'a> {
    Update { lobby: &'a lobby::Lobby },
    Start { url: String },
    Error { message: String },
}

#[derive(Serialize)]
#[serde(tag = "action")]
pub enum TurnResponse<'a> {
    Update {
        board: &'a board::Board,
    },
    GameOver {
        board: &'a board::Board,
        winner: String,
    },
    Error {
        message: String,
    },
}

enum GameStatus {
    Ongoing,
    Winner(String),
    EveryoneLoses,
}

type WebsocketSender = tokio::sync::mpsc::UnboundedSender<warp::ws::Message>;

pub struct AppState {
    games: HashMap<i64, GameManager>,
    conn: rusqlite::Connection,
    lobbies: HashMap<String, lobby::Lobby>,
    // Room -> Username -> Sender
    websockets: HashMap<String, HashMap<String, WebsocketSender>>,
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

#[derive(Debug)]
struct NotHostError;
impl fmt::Display for NotHostError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Only the host can start the game")
    }
}
impl error::Error for NotHostError {}

#[derive(Debug)]
struct NotYourTurnError;
impl fmt::Display for NotYourTurnError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Not your turn")
    }
}
impl error::Error for NotYourTurnError {}

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
            lobbies: HashMap::new(),
            websockets: HashMap::new(),
        })
    }

    pub fn new_lobby(&mut self, username: String) -> String {
        loop {
            let code = lobby::generate_lobby_code();
            if let Entry::Vacant(v) = self.lobbies.entry(code) {
                let copy = v.key().to_owned();
                v.insert(lobby::Lobby::new(username));
                return copy;
            }
        }
    }

    fn new_game_helper(
        &mut self,
        lobby_code: &str,
        username: &str,
    ) -> Result<i64> {
        let mut lobby =
            self.lobbies.remove(lobby_code).ok_or("No such lobby")?;
        match lobby.run_pregame_checks(username) {
            Ok(()) => {}
            Err(e) => {
                let ret = e.into();
                self.lobbies.insert(lobby_code.to_owned(), lobby);
                return Err(ret);
            }
        }
        lobby.prepare_for_game();
        let now = Utc::now();
        self.conn.execute(
            "INSERT INTO games (start_time, player_ids) VALUES (?1, ?2)",
            [
                now.to_rfc3339(),
                serde_json::to_string(&lobby.player_names())?,
            ],
        )?;
        let game_id = self.conn.last_insert_rowid();
        let mut rng = rand::thread_rng();
        let mut gm = GameManager::new(&mut rng);
        for (user, position) in lobby.into_seated_players() {
            gm.register_player(user, position)?;
        }
        self.games.insert(game_id, gm);
        Ok(game_id)
    }

    pub fn new_game(&mut self, lobby_code: &str, username: &str) {
        match self.new_game_helper(lobby_code, username) {
            Ok(game_id) => {
                let msg = serde_json::to_string(&LobbyResponse::Start {
                    url: format!("/game?id={}", game_id),
                })
                .unwrap();
                self.broadcast_to_room(msg, lobby_code, None);
            }
            Err(e) => {
                let msg = serde_json::to_string(&LobbyResponse::Error {
                    message: e.to_string(),
                })
                .unwrap();
                self.send_to_user(msg, lobby_code, username);
            }
        };
    }

    pub fn lobby(&self, lobby_code: &str) -> Option<&lobby::Lobby> {
        self.lobbies.get(lobby_code)
    }

    fn take_seat_helper(
        &mut self,
        lobby_code: &str,
        seat_idx: i8,
        username: &str,
    ) -> Result<&lobby::Lobby> {
        let lobby = self.lobbies.get_mut(lobby_code).ok_or("No such lobby")?;
        lobby.take_seat(seat_idx, username.to_owned())?;
        Ok(lobby)
    }

    pub fn take_seat(
        &mut self,
        lobby_code: &str,
        seat_idx: i8,
        username: &str,
    ) {
        match self.take_seat_helper(lobby_code, seat_idx, username) {
            Ok(lobby) => {
                let msg =
                    serde_json::to_string(&LobbyResponse::Update { lobby })
                        .unwrap();
                self.broadcast_to_room(msg, lobby_code, None);
            }
            Err(e) => {
                let msg = serde_json::to_string(&LobbyResponse::Error {
                    message: e.to_string(),
                })
                .unwrap();
                self.send_to_user(msg, lobby_code, username);
            }
        };
    }

    fn resize_lobby_helper(
        &mut self,
        lobby_code: &str,
        new_size: usize,
        username: &str,
    ) -> Result<&lobby::Lobby> {
        let lobby = self.lobbies.get_mut(lobby_code).ok_or("No such lobby")?;
        if lobby.host() != username {
            return Err(NotHostError.into());
        }
        lobby.resize(new_size)?;
        Ok(lobby)
    }

    pub fn resize_lobby(
        &mut self,
        lobby_code: &str,
        new_size: usize,
        username: &str,
    ) {
        match self.resize_lobby_helper(lobby_code, new_size, username) {
            Ok(lobby) => {
                let msg =
                    serde_json::to_string(&LobbyResponse::Update { lobby })
                        .unwrap();
                self.broadcast_to_room(msg, lobby_code, None);
            }
            Err(e) => {
                let msg = serde_json::to_string(&LobbyResponse::Error {
                    message: e.to_string(),
                })
                .unwrap();
                self.send_to_user(msg, lobby_code, username);
            }
        };
    }

    pub fn game(&self, game_id: i64) -> Option<&GameManager> {
        self.games.get(&game_id)
    }

    fn take_turn_helper(
        &mut self,
        params: TurnParams,
        username: &str,
    ) -> Result<(&board::Board, GameStatus)> {
        let game = self
            .games
            .get_mut(&params.game_id)
            .ok_or("Invalid game ID")?;
        if game.current_player().username != username {
            return Err(NotYourTurnError.into());
        }
        let num_alive = game.take_turn(params.idx, params.facing);
        if num_alive >= 2 {
            return Ok((&game.board, GameStatus::Ongoing));
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
                params.game_id.to_string(),
            ],
        )?;
        let players_json = self.conn.query_row(
            "SELECT player_ids FROM games WHERE id = ?1 LIMIT 1",
            [params.game_id.to_string()],
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
        Ok((
            &game.board,
            if num_alive == 1 {
                GameStatus::Winner(game.current_player().username.clone())
            } else {
                GameStatus::EveryoneLoses
            },
        ))
    }

    pub fn take_turn(&mut self, params: TurnParams, username: &str) {
        let room = &params.game_id.to_string();
        match self.take_turn_helper(params, username) {
            Ok((board, status)) => {
                let resp = match status {
                    GameStatus::Ongoing => TurnResponse::Update { board },
                    GameStatus::Winner(winner) => {
                        TurnResponse::GameOver { winner, board }
                    }
                    GameStatus::EveryoneLoses => TurnResponse::GameOver {
                        winner: "".to_owned(),
                        board,
                    },
                };
                let msg = serde_json::to_string(&resp).unwrap();
                self.broadcast_to_room(msg, room, None);
            }
            Err(e) => {
                let msg = serde_json::to_string(&TurnResponse::Error {
                    message: e.to_string(),
                })
                .unwrap();
                self.send_to_user(msg, room, username);
            }
        }
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
            "INSERT INTO players (username, hashed_password, num_games)
            VALUES (?, ?, 0)",
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

    pub fn add_user_to_room(
        &mut self,
        room: &str,
        username: &str,
        tx: WebsocketSender,
    ) {
        self.websockets
            .entry(room.to_owned())
            .or_insert_with(HashMap::new)
            .insert(username.to_owned(), tx);
    }

    pub fn remove_user_from_room(&mut self, room: &str, username: &str) {
        if let Some(users) = self.websockets.get_mut(room) {
            users.remove(username);
            if users.is_empty() {
                self.websockets.remove(room);
            }
        }
    }

    pub fn broadcast_to_room(
        &self,
        msg: String,
        room: &str,
        sender: Option<&str>,
    ) {
        for (user, tx) in self.websockets[room].iter() {
            if sender != Some(user) {
                // If the recipient disconnected, that's not our problem.
                let _ = tx.send(warp::ws::Message::text(msg.clone()));
            }
        }
    }

    fn send_to_user(&self, msg: String, room: &str, user: &str) {
        let _ = self.websockets[room][user].send(warp::ws::Message::text(msg));
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
