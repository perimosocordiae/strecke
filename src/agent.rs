use crate::board::{Board, Position, StepResult};
use crate::game::GameManager;
use crate::tiles::{Direction, Tile};
use log::info;

pub trait Agent {
    fn choose_action(&self, game: &GameManager) -> (usize, Direction);
}

pub fn create_agent(difficulty: usize) -> Box<dyn Agent + Send> {
    match difficulty {
        0 => Box::<AvoidSuddenDeathAgent>::default(),
        1 => Box::<XenophobeAgent>::default(),
        _ => Box::<LookaheadAgent>::default(),
    }
}

// Simple tile selection function that only tries to avoid immediate death.
#[derive(Default)]
pub struct AvoidSuddenDeathAgent;
impl Agent for AvoidSuddenDeathAgent {
    fn choose_action(&self, game: &GameManager) -> (usize, Direction) {
        let my_pos = game.current_player_pos();
        let me = game.current_player();
        assert!(!me.tiles_in_hand.is_empty());
        let name = &me.username;
        if let Some((i, dir)) =
            any_safe_move(&game.board, my_pos, &me.tiles_in_hand)
        {
            info!("{name}: Found safe tile {i} facing {dir:?}");
            return (i, dir);
        }
        // Fallback: no safe tile to play.
        info!("{name}: No safe tile to play, playing arbitrary tile!");
        (0, Direction::North)
    }
}

// Agent that looks one move ahead for safe plays.
#[derive(Default)]
pub struct LookaheadAgent;
impl Agent for LookaheadAgent {
    fn choose_action(&self, game: &GameManager) -> (usize, Direction) {
        let my_pos = game.current_player_pos();
        let me = game.current_player();
        let name = &me.username;
        let moves = all_safe_moves(&game.board, my_pos, &me.tiles_in_hand);
        if moves.is_empty() {
            // No safe moves, play arbitrary tile.
            info!("{name}: No safe tile to play, playing arbitrary tile!");
            return (0, Direction::North);
        }
        let backup = moves[0];
        // For each safe move, check that we have at least one safe move next turn.
        for (tile_idx, dir) in moves {
            let mut sim_board = game.board.clone();
            sim_board.play_tile(
                me.board_index,
                &me.tiles_in_hand[tile_idx],
                dir,
            );
            let pos = sim_board.players[me.board_index].last().unwrap();
            let mut tiles = me.tiles_in_hand.clone();
            tiles.swap_remove(tile_idx);
            if any_safe_move(&sim_board, pos, &tiles).is_some() {
                info!(
                    "{name}: Found safe tile {tile_idx} facing {dir:?} with safe follow-up"
                );
                return (tile_idx, dir);
            }
        }
        // No safe follow-up moves, just play the first safe move.
        let (tile_idx, dir) = backup;
        info!(
            "{name}: No safe follow-up moves, but playing safe tile {tile_idx} facing {dir:?}"
        );
        (tile_idx, dir)
    }
}

// Agent that tries to avoid landing near other players.
#[derive(Default)]
pub struct XenophobeAgent;
impl Agent for XenophobeAgent {
    fn choose_action(&self, game: &GameManager) -> (usize, Direction) {
        let my_pos = game.current_player_pos();
        let me = game.current_player();
        let name = &me.username;
        if let Some((i, dir)) =
            all_safe_moves(&game.board, my_pos, &me.tiles_in_hand)
                .into_iter()
                .max_by_key(|&(tile_idx, dir)| {
                    // Simulate the move to see where players end up.
                    let mut sim_board = game.board.clone();
                    sim_board.play_tile(
                        me.board_index,
                        &me.tiles_in_hand[tile_idx],
                        dir,
                    );
                    let pos = sim_board.players[me.board_index].last().unwrap();
                    // Find the nearest living player to the new position.
                    sim_board
                        .players
                        .iter()
                        .enumerate()
                        .filter_map(|(idx, trail)| {
                            if idx == me.board_index {
                                return None;
                            }
                            let other_pos = trail.last()?;
                            if !other_pos.alive {
                                return None;
                            }
                            Some(pos.l1_distance(other_pos))
                        })
                        .min()
                        .unwrap_or(i32::MAX)
                })
        {
            info!("{name}: Found safe tile {i} facing {dir:?}");
            return (i, dir);
        }
        // Fallback: no safe tile to play.
        info!("{name}: No safe tile to play, playing arbitrary tile!");
        (0, Direction::North)
    }
}

fn any_safe_move(
    board: &Board,
    start_pos: &Position,
    tiles: &[Tile],
) -> Option<(usize, Direction)> {
    for (i, tile) in tiles.iter().enumerate() {
        for dir in Direction::all() {
            let end_pos = follow_path(board, start_pos, tile, dir);
            if end_pos.alive {
                return Some((i, dir));
            }
        }
    }
    None
}

fn all_safe_moves(
    board: &Board,
    start_pos: &Position,
    tiles: &[Tile],
) -> Vec<(usize, Direction)> {
    tiles
        .iter()
        .enumerate()
        .flat_map(|(i, tile)| {
            tile.unique_facings(start_pos.port).into_iter().filter_map(
                move |dir| {
                    let end_pos = follow_path(board, start_pos, tile, dir);
                    if end_pos.alive { Some((i, dir)) } else { None }
                },
            )
        })
        .collect()
}

fn follow_path(
    board: &Board,
    start_pos: &Position,
    played_tile: &Tile,
    dir: Direction,
) -> Position {
    // Simulate the given tile being played.
    let initial_move = start_pos.next_tile_position();
    let tile_coords = (initial_move.row, initial_move.col);
    let virtual_tile = Some((tile_coords, *played_tile, dir));

    // First, traverse the played tile.
    let mut current_pos = start_pos.clone();

    loop {
        match board.step(&current_pos, virtual_tile) {
            StepResult::Moved(new_pos) => current_pos = new_pos,
            StepResult::OffBoard(dead_pos) => return dead_pos,
            StepResult::Blocked(end_pos) => return end_pos,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::tiles::{Port, all_tiles};

    #[test]
    fn test_follow_path_basic() {
        let board = Board::default();
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
