mod board;
mod manager;
mod tiles;

use std::sync::Arc;
use tokio::sync::Mutex;
use warp::Filter;

type Database = Arc<Mutex<manager::GameManager>>;

#[tokio::main]
async fn main() {
    // Run with RUST_LOG=info to see log messages.
    pretty_env_logger::init();

    let mut rng = rand::thread_rng();
    let mut gm = manager::GameManager::new(&mut rng);

    // Start demo code to exercise the game logic--------------------
    gm.register_player(
        "CJ",
        board::Position {
            row: 2,
            col: 6,
            port: tiles::Port::G,
        },
    )
    .unwrap();
    gm.rotate_tile(0, 0);
    let fake_json = serde_json::to_string(&manager::PlayerMove {
        tile_index: 0,
        row: 2,
        col: 5,
    })
    .unwrap();
    gm.take_turn(&serde_json::from_str(&fake_json).unwrap())
        .unwrap();
    // End demo code ------------------------------------------------

    let db: Database = Arc::new(Mutex::new(gm));

    let index = warp::path::end().and(warp::fs::file("./static/index.html"));
    let files = warp::path("static").and(warp::fs::dir("./static/"));

    // GET /board => JSON
    let board = warp::path!("board")
        .map(move || db.clone())
        .and_then(get_board_json);

    let routes = warp::get().and(index.or(files).or(board));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

async fn get_board_json(
    db: Database,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    let gm = db.lock().await;
    Ok(warp::reply::json(&gm.board))
}
