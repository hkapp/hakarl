use chess::{Board, BoardStatus, ChessMove, Color};
use core::ops::Not;
use crate::pgn;

pub mod random;

pub trait ChessPlayer {
    fn pick_move(&mut self, board: &Board) -> ChessMove;
}

pub fn play_game<P1: ChessPlayer, P2: ChessPlayer>(mut white: P1, mut black: P2) {
    let mut board = Board::default();
    let mut move_list = Vec::new();
    let max_moves = 200;

    while board.status() == BoardStatus::Ongoing && move_list.len() < max_moves {
        let mv = match board.side_to_move() {
            Color::White => white.pick_move(&board),
            Color::Black => black.pick_move(&board)
        };
        move_list.push(mv);
        board = board.make_move_new(mv);
        println!("Board value is now: {}", crate::eval::classic_eval(&board, board.side_to_move()));
    }

    println!("{}", pgn::basic_pgn(&move_list));
    print_end_of_game(&board);
}

pub fn play_random_game() {
    let white = random::random_player();
    let black = random::random_player();
    play_game(white, black);
}

fn print_end_of_game(board: &Board) {
    match board.status() {
        BoardStatus::Checkmate => println!("Player {:?} wins!", board.side_to_move().not()),
        BoardStatus::Stalemate => println!("The game is a draw!"),
        BoardStatus::Ongoing   => println!("Maximum number of moves reached")
    }
}
