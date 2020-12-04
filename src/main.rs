/* This needs to be the first import to let the next modules access the macros
 * defined in 'logging'
 */
#[macro_use]
mod logging;

mod pgn;
mod eval;
mod play;
mod utils;

use chess::{Board, BoardStatus, Color, ChessMove};
use play::Game;
use std::fs::File;
use std::path::Path;
use std::io::Write;
use std::io;
use std::time::Duration;

#[allow(unused_variables)]
fn main() {
    //let white = play::montecarlo::basic_monte_carlo1();
    //let white = play::exhaustive::exhaustive_search_player(2);
    //let black = play::astar::asprl::parallel_player(Duration::from_millis(30), 4);
    let white = play::astar::astar_player(Duration::from_millis(30));
    let black = play::evaldriven::classic_eval_player();

    let log_level = logging::LogLevel::Debug;

    /*play_a_game(white, black, log_level);*/
    explain_move_from_prev_game(
        white,
        &Path::new("games/debug_astar_201115/debug_move_17.pgn"),
        Color::White,
        17,
        log_level);
}

/***********  PLAY **********/

const LOG_FILE_PATH: &str = "games/last_game.log";
const PGN_FILE_PATH: &str = "games/last_game.pgn";

#[allow(dead_code)]
fn play_a_game<P1, P2>(mut white: P1, mut black: P2, log_level: logging::LogLevel)
    where
        P1: play::ChessPlayer,
        P2: play::ChessPlayer
{
    let mut game_logger = logging::log_to_file(&Path::new(LOG_FILE_PATH), log_level)
                                    .expect(&format!("Couldn't open file {}", LOG_FILE_PATH));

    let game = play::play_game(&mut white, &mut black, &mut game_logger);

    /* Print the move list in pgn format */
    let pgn_format = pgn::basic_pgn(&game.moves);
    println!("{}", pgn_format);

    let mut pgn_file = open_file_for_write(&Path::new(PGN_FILE_PATH));
    let pgn_written = write!(pgn_file, "{}", pgn_format);

    /* Print the result of the game */
    print_end_of_game(&game);
    println!("The explanation of the moves can be found in '{}'", LOG_FILE_PATH);
    match pgn_written {
        Ok(_)       => println!("The pgn can also be found in '{}'", PGN_FILE_PATH),
        Err(reason) => println!("The pgn could not be written to a file: {}", reason),
    };
}

fn open_file_for_write(path: &Path) -> File {
    match File::create(path) {
        Ok(file)    => file,
        Err(reason) => panic!("Couldn't open {}: {}", path.display(), reason)
    }
}

fn print_end_of_game(game: &Game) {
    match game.final_board.status() {
        BoardStatus::Checkmate => println!("Player {:?} wins!", game.winner().unwrap()),
        BoardStatus::Stalemate => println!("The game is a draw!"),
        BoardStatus::Ongoing   => println!("Maximum number of moves reached")
    }
}

/***********  EXPLAIN **********/

fn find_move_in_game(game: &play::Game, player: Color, turn: u16) -> Option<(Board, ChessMove)> {
    let mut curr_board = game.init_board;
    let mut curr_turn = 1;
    for mv_ref in game.moves.iter() {
        let curr_player = curr_board.side_to_move();
        if curr_turn == turn && curr_player == player {
            return Some((curr_board, *mv_ref));
        }
        else {
            curr_board = curr_board.make_move_new(*mv_ref);
            if curr_player == Color::Black {
                curr_turn += 1;
            }
        }
    }
    None
}

fn explain_move_from_prev_game(
    mut player:   play::astar::AStar,
    pgn_to_load:  &Path,
    debug_player: Color,
    turn:         u16,
    log_level:    logging::LogLevel)
{
    //let mut logger = logging::log_to_file(&Path::new(LOG_FILE_PATH), log_level)
                                    //.expect(&format!("Couldn't open file {}", LOG_FILE_PATH));
    let game_pgn = std::fs::read_to_string(pgn_to_load).unwrap();
    let full_game = pgn::read_pgn(&game_pgn).unwrap();
    let (debug_board, debug_mv) = find_move_in_game(&full_game, debug_player, turn).unwrap();

    let mut logger = logging::log_to(io::stdout(), log_level);
    let max_retries = 2000;
    for try_count in 0..max_retries {
        use play::DebugPlayer;
        let search_tree = player.compute_move(&debug_board, &mut logger);
        let played_mv = player.best_move(&search_tree);

        //let played_mv = player.write_res_tree(&debug_board, &mut logger, &mut dot_file).unwrap();

        if played_mv == debug_mv {
            use play::astar;
            println!("Found the same move! ({})", debug_mv);
            let dot_graph = astar::build_dot_graph_from(&player, &search_tree);

            let dot_path = pgn_to_load.with_extension("dot");
            let mut dot_file = open_file_for_write(&dot_path);
            match dot_graph.write_to(&mut dot_file) {
                Ok(_) => println!("Written to {:?}", dot_path),
                Err(e) => println!("{:?}", e)
            }
                //.map(|_| finalize(&search_tree, self.eval, self.time_budget, logger))
            return;
        }
        else if try_count < max_retries {
            println!("Not the same move, retrying... ({})", try_count+1);
        }
    }
    println!("Did not manage to find the same move, giving up");
    //assert_eq!(played_mv, debug_mv);
}
