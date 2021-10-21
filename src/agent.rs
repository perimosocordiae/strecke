use crate::board::Board;
use crate::tiles::{Direction, Tile};

pub fn select_tile(
    _board: &Board,
    board_indices: &[usize],
    _hand: &[Tile],
) -> (usize, Direction) {
    let _my_bidx = board_indices[0];
    // TODO: do something smart here.
    (0, Direction::North)
}
