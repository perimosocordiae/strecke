mod board;
mod game;
mod tiles;
mod webapp;

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

type Database = Arc<Mutex<webapp::AppState>>;

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=info to see log messages.
    pretty_env_logger::init();

    let db: Database =
        Arc::new(Mutex::new(webapp::AppState::new("strecke.db").unwrap()));

    let index = warp::path::end().and(warp::fs::file("./static/index.html"));
    let game = warp::path("game").and(warp::fs::file("./static/game.html"));
    let files = warp::path("static").and(warp::fs::dir("./static/"));

    // GET /board => JSON
    let db1 = db.clone();
    let board = warp::path!("board")
        .map(move || db1.clone())
        .and_then(get_board_json);

    // GET /hand => JSON
    let db2 = db.clone();
    let hand = warp::path!("hand")
        .map(move || db2.clone())
        .and_then(get_hand_json);

    // POST /play/$tile_idx => OK
    let db3 = db.clone();
    let play = warp::path!("play" / usize)
        .and(warp::any().map(move || db3.clone()))
        .and_then(play_tile);

    // POST /rotate/$player_idx/$tile_idx => OK
    let rotate = warp::path!("rotate" / usize / usize)
        .and(warp::any().map(move || db.clone()))
        .and_then(rotate_tile);

    let gets = warp::get().and(index.or(game).or(files).or(board).or(hand));
    let posts = warp::post().and(play.or(rotate));
    let routes = gets.or(posts);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn get_board_json(
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let app = db.lock().await;
    Ok(warp::reply::json(&app.gm.board))
}

async fn get_hand_json(
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let app = db.lock().await;
    Ok(warp::reply::json(app.gm.current_player()))
}

async fn play_tile(
    idx: usize,
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut app = db.lock().await;
    Ok(match app.gm.take_turn(idx) {
        0 => "Everyone loses",
        1 => "Somebody won",
        _ => "OK",
    })
}

async fn rotate_tile(
    player_idx: usize,
    tile_idx: usize,
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let mut app = db.lock().await;
    app.gm.rotate_tile(player_idx, tile_idx);
    Ok("OK")
}
