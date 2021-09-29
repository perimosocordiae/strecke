mod board;
mod game;
mod lobby;
mod settings;
mod tiles;
mod webapp;

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::http::{header, Response, StatusCode, Uri};
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

    // GET /check_login
    let check_login = warp::path("check_login").and(needs_cookie).map(|_| "OK");

    // POST /new_game => JSON
    let new_lobby = warp::path("new_lobby")
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(do_new_lobby);

    // POST /new_game => JSON
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

    // POST /play/$game_id/$tile_idx => OK
    let play = warp::path!("play" / i64 / usize)
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(play_tile);

    // POST /rotate/$game_id/$tile_idx => OK
    let rotate = warp::path!("rotate" / i64 / usize)
        .and(db_getter.clone())
        .and(needs_cookie)
        .and_then(rotate_tile);

    let gets = warp::get().and(
        index
            .or(game)
            .or(static_files)
            .or(board)
            .or(hand)
            .or(check_login),
    );
    let posts = warp::post().and(
        play.or(rotate)
            .or(login)
            .or(register)
            .or(logout)
            .or(new_lobby)
            .or(new_game),
    );
    let routes = gets.or(posts);

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
    let mut app = db.lock().await;
    Ok(match app.new_game(lobby_code, username) {
        Ok(game_id) => Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("/game?id={}", game_id))
            .body("".to_owned()),
        Err(e) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(e.to_string()),
    })
}

async fn do_new_lobby(
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let mut app = db.lock().await;
    Ok(match app.new_lobby(username) {
        Ok(lobby_code) => Response::builder()
            .status(StatusCode::SEE_OTHER)
            .header(header::LOCATION, format!("/lobby?code={}", lobby_code))
            .body("".to_owned()),
        Err(e) => Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(e.to_string()),
    })
}

async fn get_board_json(
    game_id: i64,
    db: Database,
    _username: String,
) -> WarpResult<impl warp::Reply> {
    let app = db.lock().await;
    Ok(warp::reply::json(&app.game(game_id).board))
}

async fn get_hand_json(
    game_id: i64,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let app = db.lock().await;
    Ok(match app.game(game_id).get_player(&username) {
        Some(p) => warp::reply::json(p),
        None => warp::reply::json(&"Player not found."),
    })
}

async fn play_tile(
    game_id: i64,
    idx: usize,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let mut app = db.lock().await;
    Ok(match app.take_turn(game_id, &username, idx) {
        // TODO: when game is over, redirect to an endgame page
        Ok(msg) => msg.to_owned(),
        Err(e) => e.to_string(),
    })
}

async fn rotate_tile(
    game_id: i64,
    tile_idx: usize,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let mut app = db.lock().await;
    Ok(match app.rotate_tile(game_id, &username, tile_idx) {
        Ok(msg) => msg.to_owned(),
        Err(e) => e.to_string(),
    })
}
