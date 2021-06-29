use crate::board::Board;
use crate::tiles::Tile;

pub struct GameManager {
    board: Board,
    tile_stack: Vec<Tile>,
}  