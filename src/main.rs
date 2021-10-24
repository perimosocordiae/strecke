mod agent;
mod board;
mod game;
mod lobby;
mod settings;
mod tiles;
mod webapp;

use futures::{SinkExt, StreamExt, TryFutureExt};
use std::sync::Arc;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::http::{header, Response, StatusCode, Uri};
use warp::ws::WebSocket;
use warp::Filter;

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;

type Database = Arc<Mutex<webapp::AppState>>;

lazy_static! {
    static ref CONFIG: settings::Settings = settings::Settings::new().unwrap();
}

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=info to see log messages.
    pretty_env_logger::init();

    let app = webapp::AppState::new(&CONFIG.db.name).unwrap();
    let db: Database = Arc::new(Mutex::new(app));
    let db_getter = warp::any().map(move || Arc::clone(&db));
    let needs_cookie = warp::cookie(&CONFIG.cookie.name).and_then(
        move |token: String| async move {
            match webapp::decode_jwt(&token, CONFIG.cookie.decoder()) {
                Ok(username) => Ok(username),
                Err(e) => {
                    error!("JWT decode failed: {:?}", e);
                    Err(warp::reject())
                }
            }
        },
    );

    let index = warp::path::end().and(warp::fs::file("./static/index.html"));
    let game = warp::path("game").and(warp::fs::file("./static/game.html"));
    let lobby = warp::path("lobby").and(warp::fs::file("./static/lobby.html"));
    let static_files = warp::path("static").and(warp::fs::dir("./static/"));
    // .with(warp::log("access"));

    // POST /login, /register, /logout
    let login = warp::path("login")
        .and(warp::body::json())
        .and(db_getter.clone())
        .and_then(do_login);
    let register = warp::path("register")
        .and(warp::body::json())
        .and(db_getter.clone())
        .and_then(do_register);
    let logout = warp::path("logout")
        .map(|| warp::redirect::see_other(Uri::from_static("/")))
        .map(|reply| {
            warp::reply::with_header(
                reply,
                header::SET_COOKIE,
                format!(
                    "{}=; expires=Thu, 01 Jan 1970 00:00:00 GMT;",
                    CONFIG.cookie.name
                ),
            )
        });

    // GET /check_login => username
    let check_login = warp::path("check_login")
        .and(needs_cookie)
        .map(|username| username);

    // POST /new_lobby => JSON
    let new_lobby = warp::path("new_lobby")
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(do_new_lobby);

    // POST /new_game/$code => JSON
    let new_game = warp::path!("new_game" / String)
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(do_new_game);

    // GET /board/$game_id => JSON
    let board = warp::path!("board" / i64)
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(get_board_json);

    // GET /hand/$game_id => JSON
    let hand = warp::path!("hand" / i64)
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(get_hand_json);

    // GET /lobby/$code => JSON
    let lobby_data = warp::path!("lobby_data" / String)
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(get_lobby_json);

    // POST /play => OK
    let play = warp::path("play")
        .and(warp::body::json())
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(play_tile);

    // POST /lobby_seat/$code/$seat_idx
    let lobby_seat = warp::path!("lobby_seat" / String / i8)
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(take_seat);

    // GET /ws => websocket
    let ws = warp::path!("ws" / String)
        .and(warp::ws())
        .and(db_getter.clone())
        .and(needs_cookie)
        .map(|room, ws: warp::ws::Ws, db, username| {
            ws.on_upgrade(move |socket| {
                new_connection(socket, db, room, username)
            })
        });

    let gets = warp::get().and(
        index
            .or(game)
            .or(lobby)
            .or(static_files)
            .or(board)
            .or(hand)
            .or(lobby_data)
            .or(check_login),
    );
    let posts = warp::post().and(
        play.or(lobby_seat)
            .or(login)
            .or(register)
            .or(logout)
            .or(new_lobby)
            .or(new_game),
    );

    let routes = gets.or(posts).or(ws);

    warp::serve(routes)
        .run(([0, 0, 0, 0], CONFIG.server.port))
        .await;
}

type WarpResult<T> = Result<T, std::convert::Infallible>;

async fn do_login(
    creds: webapp::UserCredentials,
    db: Database,
) -> WarpResult<impl warp::Reply> {
    let app = db.lock().await;
    Ok(match app.check_login(creds, CONFIG.cookie.encoder()) {
        Ok(access_token) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::SET_COOKIE,
                format!(
                    "{}={}; HttpOnly; SameSite=Strict",
                    CONFIG.cookie.name, access_token
                ),
            )
            .body("Logged in".to_owned()),
        Err(e) => {
            error!("Failed login: {:?}", e);
            Response::builder()
                .status(StatusCode::UNAUTHORIZED)
                .body(e.to_string())
        }
    })
}

async fn do_register(
    creds: webapp::UserCredentials,
    db: Database,
) -> WarpResult<impl warp::Reply> {
    let mut app = db.lock().await;
    Ok(match app.sign_up(creds, CONFIG.cookie.encoder()) {
        Ok(access_token) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::SET_COOKIE,
                format!(
                    "{}={}; HttpOnly; SameSite=Strict",
                    CONFIG.cookie.name, access_token
                ),
            )
            .body("Created user".to_owned()),
        Err(e) => {
            error!("Failed to register: {:?}", e);
            Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(e.to_string())
        }
    })
}

async fn do_new_game(
    lobby_code: String,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    db.lock().await.new_game(&lobby_code, &username);
    info!(
        "Started game from lobby {} for user {}",
        &lobby_code, &username
    );
    Ok("OK")
}

async fn do_new_lobby(
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let mut app = db.lock().await;
    let lobby_code = app.new_lobby(username.clone());
    info!("Made lobby {} for user {}", &lobby_code, &username);
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, format!("/lobby?code={}", lobby_code))
        .body("".to_owned()))
}

async fn get_board_json(
    game_id: i64,
    db: Database,
    _username: String,
) -> WarpResult<impl warp::Reply> {
    let app = db.lock().await;
    Ok(match app.game(game_id) {
        Some(game) => warp::reply::json(&game.board),
        None => warp::reply::json(&"Game not found."),
    })
}

async fn get_hand_json(
    game_id: i64,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let app = db.lock().await;
    Ok(match app.game(game_id) {
        Some(game) => match game.get_player(&username) {
            Some(p) => warp::reply::json(p),
            None => warp::reply::json(&"Player not found."),
        },
        None => warp::reply::json(&"Game not found."),
    })
}

async fn get_lobby_json(
    lobby_code: String,
    db: Database,
    _username: String,
) -> WarpResult<impl warp::Reply> {
    let app = db.lock().await;
    Ok(match app.lobby(&lobby_code) {
        Some(lobby) => warp::reply::json(lobby),
        None => warp::reply::json(&"No such lobby"),
    })
}

async fn play_tile(
    params: webapp::TurnParams,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    db.lock().await.take_turn(params, &username);
    Ok("OK")
}

async fn take_seat(
    lobby_code: String,
    seat_idx: i8,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    db.lock().await.take_seat(&lobby_code, seat_idx, &username);
    Ok("OK")
}

async fn new_connection(
    ws: WebSocket,
    db: Database,
    room: String,
    username: String,
) {
    info!("Got new ws connection from {} in {}", &username, &room);
    let (mut ws_tx, mut ws_rx) = ws.split();
    // Set up a channel to buffer messages.
    let (tx, rx) = mpsc::unbounded_channel();
    let mut rx = UnboundedReceiverStream::new(rx);
    tokio::task::spawn(async move {
        while let Some(message) = rx.next().await {
            ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    error!("WS send error: {}", e);
                })
                .await;
        }
    });
    // Save a handle to this user's websocket.
    db.lock().await.add_user_to_room(&room, &username, tx);
    // Handle incoming messages from this user.
    while let Some(result) = ws_rx.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("WS error for user {}: {}", &username, e);
                break;
            }
        };
        if let Ok(text) = msg.to_str() {
            info!("Broadcasting from {} -> {}: {}", &username, &room, &text);
            db.lock().await.broadcast_to_room(
                text.to_owned(),
                &room,
                Some(&username),
            );
        }
    }
    // The above loop only ends when the user disconnects.
    info!("Disconnected WS for {}", &username);
    db.lock().await.remove_user_from_room(&room, &username);
}
