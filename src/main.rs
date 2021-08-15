mod board;
mod game;
mod tiles;
mod webapp;

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::http::StatusCode;
use warp::Filter;

#[macro_use]
extern crate log;

type Database = Arc<Mutex<webapp::AppState>>;

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=info to see log messages.
    pretty_env_logger::init();

    let mut app = webapp::AppState::new("strecke.db").unwrap();
    let _game_id = app.new_game().unwrap(); // XXX demo code
    let db: Database = Arc::new(Mutex::new(app));
    let db_getter = warp::any().map(move || Arc::clone(&db));

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
        .and_then(get_board_json);

    // GET /hand/$game_id => JSON
    let hand = warp::path!("hand" / i64)
        .and(db_getter.clone())
        .and_then(get_hand_json);

    // POST /play/$game_id/$tile_idx => OK
    let play = warp::path!("play" / i64 / usize)
        .and(db_getter.clone())
        .and_then(play_tile);

    // POST /rotate/$game_id/$tile_idx => OK
    let rotate = warp::path!("rotate" / i64 / usize)
        .and(db_getter.clone())
        .and_then(rotate_tile);

    let gets = warp::get().and(index.or(game).or(files).or(board).or(hand));
    let posts = warp::post().and(play.or(rotate).or(login));
    let routes = gets.or(posts);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn do_login(
    creds: webapp::UserCredentials,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let app = db.lock().await;
    Ok(match app.check_login(creds) {
        Ok(is_auth) => {
            if is_auth {
                StatusCode::OK
            } else {
                StatusCode::UNAUTHORIZED
            }
        }
        Err(e) => {
            error!("Failed login: {:?}", e);
            StatusCode::BAD_REQUEST
        }
    })
}

async fn do_register(
    creds: webapp::UserCredentials,
    db: Database,
) -> Result<impl warp::Reply, warp::Rejection> {
    let mut app = db.lock().await;
    app.sign_up(creds).unwrap();
    Ok("OK")
}

async fn get_board_json(
    game_id: i64,
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let app = db.lock().await;
    Ok(warp::reply::json(&app.game(game_id).board))
}

async fn get_hand_json(
    game_id: i64,
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let app = db.lock().await;
    Ok(warp::reply::json(app.game(game_id).current_player()))
}

async fn play_tile(
    game_id: i64,
    idx: usize,
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut app = db.lock().await;
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
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut app = db.lock().await;
    app.mut_game(game_id).rotate_tile(0, tile_idx);
    Ok("OK")
}
