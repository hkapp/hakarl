use chess::{Board, MoveGen, BoardStatus, ChessMove, Color};
use rand::seq::IteratorRandom;
use rand::Rng;
use core::ops::Not;

mod pgn;

fn main() {
    play_random_game();
}

fn pick_random_move<R: Rng>(board: &Board, rng: &mut R) -> ChessMove {
    let movegen = MoveGen::new_legal(&board);
    match movegen.choose(rng) {
        Some(mv) => mv,
        None     => panic!("Couldn't find a move!'")
    }
}

fn play_random_move<R: Rng>(board: &Board, rng: &mut R) -> Board {
    let mv = pick_random_move(board, rng);
    println!("{}", mv);
    board.make_move_new(mv)
}

fn print_end_of_game(board: &Board) {
    match board.status() {
        BoardStatus::Checkmate => println!("Player {:?} wins!", board.side_to_move().not()),
        BoardStatus::Stalemate => println!("The game is a draw!"),
        BoardStatus::Ongoing   => println!("Maximum number of moves reached")
    }
}

trait ChessPlayer {
    fn pick_move(&mut self, board: &Board) -> ChessMove;
}

struct RandomPlayer<R: Rng> {
    rng: R
}

//impl<R: Rng> RandomPlayer<R> {
    //fn new() -> RandomPlayer<rand::rngs::ThreadRng> {
        //RandomPlayer {
            //rng: rand::thread_rng()
        //}
    //}
//}

fn random_player() -> RandomPlayer<rand::rngs::ThreadRng> {
    RandomPlayer {
        rng: rand::thread_rng()
    }
}

impl<R: Rng> ChessPlayer for RandomPlayer<R> {
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        pick_random_move(board, &mut self.rng)
    }
}

fn play_game<P1: ChessPlayer, P2: ChessPlayer>(mut white: P1, mut black: P2) {
    let mut board = Board::default();
    let mut move_list = Vec::new();
    let max_moves = 200;

    while board.status() == BoardStatus::Ongoing && move_list.len() < max_moves {
        let mv = match board.side_to_move() {
            Color::White => white.pick_move(&board),
            Color::Black => black.pick_move(&board)
        };
        move_list.push(mv);
        board = board.make_move_new(mv)
    }

    println!("{}", pgn::basic_pgn(&move_list));
    print_end_of_game(&board);
}

fn play_random_game() {
    let white = random_player();
    let black = random_player();
    play_game(white, black);
    //let mut board = Board::default();
    //let mut rng = rand::thread_rng();
    //let max_moves = 200;

    //let mut move_list = Vec::new();

    //while board.status() == BoardStatus::Ongoing && move_list.len() < max_moves {
        //let mv = pick_random_move(&board, &mut rng);
        //move_list.push(mv.clone());
        //board = board.make_move_new(mv)
    //}

    //println!("{}", pgn::basic_pgn(&move_list));
    //print_end_of_game(&board);
}
