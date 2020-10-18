mod pgn;
mod eval;
mod play;

use chess::BoardStatus;
use play::Game;

fn main() {
    let mut white = play::evaldriven::classic_eval_player();
    let mut black = play::evaldriven::classic_eval_player(); //play::random::random_player();

    let game = play::play_game(&mut white, &mut black);

    /* Print the move list in pgn format */
    println!("{}", pgn::basic_pgn(&game.moves));
    /* Print the result of the game */
    print_end_of_game(&game);
}

fn print_end_of_game(game: &Game) {
    match game.final_board.status() {
        BoardStatus::Checkmate => println!("Player {:?} wins!", game.winner()),
        BoardStatus::Stalemate => println!("The game is a draw!"),
        BoardStatus::Ongoing   => println!("Maximum number of moves reached")
    }
}
