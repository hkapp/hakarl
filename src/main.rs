mod pgn;
mod eval;
mod play;
mod utils;

use chess::BoardStatus;
use play::Game;

fn main() {
    //let mut white = play::evaldriven::classic_eval_player();
    //let mut white = play::montecarlo::basic_monte_carlo1();
    //let mut white = play::evaldriven::classic_eval_player();
    let mut white = play::exhaustive::exhaustive_search_player(2);
    let mut black = play::astar::astar_player();

    let game = play::play_game(&mut white, &mut black);

    /* Print the move list in pgn format */
    println!("{}", pgn::basic_pgn(&game.moves));
    /* Print the result of the game */
    print_end_of_game(&game);
}

fn print_end_of_game(game: &Game) {
    match game.final_board.status() {
        BoardStatus::Checkmate => println!("Player {:?} wins!", game.winner().unwrap()),
        BoardStatus::Stalemate => println!("The game is a draw!"),
        BoardStatus::Ongoing   => println!("Maximum number of moves reached")
    }
}
