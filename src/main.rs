mod board;
mod game;
mod settings;
mod tiles;
mod webapp;

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::http::{header, Response, StatusCode};
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

    let mut app = webapp::AppState::new(&CONFIG.db.name).unwrap();
    let _game_id = app.new_game().unwrap(); // XXX demo code
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
    let files = warp::path("static").and(warp::fs::dir("./static/"));
    // .with(warp::log("access"));

    // XXX TODO add player auth and lookup playerIdx from that.
    let login = warp::path("login")
        .and(warp::body::json())
        .and(db_getter.clone())
        .and_then(do_login);
    let register = warp::path("register")
        .and(warp::body::json())
        .and(db_getter.clone())
        .and_then(do_register);

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

    let gets = warp::get().and(index.or(game).or(files).or(board).or(hand));
    let posts = warp::post().and(play.or(rotate).or(login).or(register));
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
    Ok(warp::reply::json(app.game(game_id).get_player(&username)))
}

async fn play_tile(
    game_id: i64,
    idx: usize,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let mut app = db.lock().await;
    if app.game(game_id).current_player().username != username {
        return Ok("not your turn");
    }
    // TODO: when the result is zero, invalidate the game (and restart?)
    Ok(match app.mut_game(game_id).take_turn(idx) {
        0 => "Everyone loses",
        1 => "Somebody won",
        _ => "OK",
    })
}

async fn rotate_tile(
    game_id: i64,
    tile_idx: usize,
    db: Database,
    username: String,
) -> WarpResult<impl warp::Reply> {
    let mut app = db.lock().await;
    app.mut_game(game_id)
        .mut_player(&username)
        .rotate_tile(tile_idx);
    Ok("OK")
}
