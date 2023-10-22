use clap::Parser;
use rand::seq::SliceRandom;
use strecke::agent;
use strecke::board;
use strecke::game::GameManager;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value_t = 1000)]
    games: usize,
    #[clap(short, long, value_delimiter = ',', default_value = "0,0,0,0")]
    agents: Vec<usize>,
}

fn main() {
    // Run with RUST_LOG=info to see log messages.
    pretty_env_logger::init();

    let args = Args::parse();
    let mut rng = rand::thread_rng();

    let num_players = args.agents.len();
    let mut positions: Vec<i8> = (0i8..48i8).collect();
    for _game_idx in 0..args.games {
        let agents = args
            .agents
            .iter()
            .map(|&i| agent::create_agent(i))
            .collect::<Vec<_>>();
        let mut game = GameManager::new(&mut rng);
        positions.shuffle(&mut rng);
        for pos in positions.iter().take(num_players) {
            game.register_player(
                format!("p{}", pos),
                board::edge_position(*pos),
            )
            .unwrap();
        }
        loop {
            let ai = &agents[game.current_player_idx];
            let (tile_idx, facing) = ai.choose_action(&game);
            if game.take_turn(tile_idx, facing).is_some() {
                break;
            }
        }

        println!(
            "{}",
            game.player_scores()
                .into_iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
    }
}
