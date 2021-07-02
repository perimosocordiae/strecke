mod board;
mod manager;
mod tiles;

extern crate pretty_env_logger;
#[macro_use]
extern crate log;

use warp::Filter;

#[tokio::main]
async fn main() {
    let mut rng = rand::thread_rng();
    let mut gm = manager::GameManager::new(&mut rng);
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

    // Run with RUST_LOG=info to see log messages.
    pretty_env_logger::init();

    let index = warp::path::end().and(warp::fs::file("./static/index.html"));

    // GET /agent/$name/ => 200 OK with body "Hello $name!"
    let hello = warp::path!("agent" / String)
        .and(warp::header("user-agent"))
        .map(|name: String, agent: String| {
            info!("Got a request from {}", name);
            format!("Hello {}! Your agent is {}", name, agent)
        });

    let routes = warp::get().and(index.or(hello));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
