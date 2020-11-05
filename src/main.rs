mod pgn;
mod eval;
mod play;
mod utils;
mod logging;

use chess::BoardStatus;
use play::Game;
use std::fs::File;
use std::path::Path;
use std::io::Write;

fn main() {
    //let mut white = play::evaldriven::classic_eval_player();
    //let mut white = play::montecarlo::basic_monte_carlo1();
    //let mut white = play::evaldriven::classic_eval_player();
    let mut white = play::exhaustive::exhaustive_search_player(2);
    let mut black = play::astar::astar_player();

    let mut game_logger = open_file_for_write(&Path::new(LOG_FILE_PATH));

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

const LOG_FILE_PATH: &str = "games/last_game.log";
const PGN_FILE_PATH: &str = "games/last_game.pgn";

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
