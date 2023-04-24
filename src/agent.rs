use crate::board::{Board, Position};
use crate::tiles::{Direction, Tile};
use log::info;

// Simple tile selection function that only tries to avoid immediate death.
pub fn select_tile(
    board: &Board,
    board_indices: &[usize],
    hand: &[Tile],
) -> (usize, Direction) {
    let my_idx = board_indices[0];
    let my_pos = board.players[my_idx].last().unwrap();
    for (i, tile) in hand.iter().enumerate() {
        for dir in Direction::all() {
            let end_pos = follow_path(board, my_pos, tile, dir);
            if end_pos.alive {
                return (i, dir);
            }
        }
    }
    // Fallback: no safe tile to play.
    info!("No safe tile to play, playing arbitrary tile!");
    (0, Direction::North)
}

// TODO: refactor this w/ Board::play_tile
fn follow_path(
    board: &Board,
    start_pos: &Position,
    played_tile: &Tile,
    dir: Direction,
) -> Position {
    // Simulate the given tile being played.
    let mut pos = start_pos.next_tile_position();
    let tile_row = pos.row;
    let tile_col = pos.col;
    pos.port = played_tile.traverse(pos.port, dir);
    // Follow the path until we fall off the board or hit an empty tile.
    loop {
        pos = pos.next_tile_position();
        match board.get_tile(&pos) {
            // Fell off the board.
            None => {
                pos.alive = false;
                return pos;
            }
            // Hit a blank grid cell.
            Some(None) => {
                if pos.row == tile_row && pos.col == tile_col {
                    // Re-traverse our initial tile (from a different port).
                    pos.port = played_tile.traverse(pos.port, dir);
                } else {
                    // We hit an empty tile.
                    return pos;
                }
            }
            // Hit an existing tile, traverse and keep looping.
            Some(Some((t, facing))) => {
                pos.port = t.traverse(pos.port, *facing);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tiles::{all_tiles, Port};

    #[test]
    fn test_follow_path_basic() {
        let board = Board::new();
        let start_pos = Position {
            row: 6,
            col: 0,
            port: Port::A,
            alive: true,
        };
        let played_tile = all_tiles()[27];
        let dir = Direction::North;
        let end_pos = follow_path(&board, &start_pos, &played_tile, dir);
        assert!(!end_pos.alive);
        assert_eq!(end_pos.row, 5);
        assert_eq!(end_pos.col, -1);
        assert_eq!(end_pos.port, Port::C);
    }
}
