mod board;
mod tiles;

fn main() {
    let mut b = board::Board::new();
    let p1 = b
        .add_player(board::Position {
            row: 2,
            col: 6,
            port: tiles::Port::G,
        })
        .unwrap();
    println!("Player {} is at {:?}", p1, b.players[p1]);

    let mut tile_pile = tiles::all_tiles();
    let x = tile_pile.pop().unwrap().rotate_left();
    println!("Player {} has {:?}", p1, x);
    let game_over = b.play_tile(p1, x, 2, 5).unwrap();
    println!(
        "After one move, player {} is at {:?} and game_over={}",
        p1, b.players[p1], game_over
    );
}
