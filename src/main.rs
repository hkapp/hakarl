use chess::{Board, MoveGen, BoardStatus, ChessMove};
use rand::seq::IteratorRandom;
use rand::Rng;

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
        BoardStatus::Checkmate => println!("Player {:?} wins!", board.side_to_move()),
        BoardStatus::Stalemate => println!("The game is a draw!"),
        BoardStatus::Ongoing   => println!("Maximum number of moves reached")
    }
}

fn play_random_game() {
    let mut board = Board::default();
    let mut rng = rand::thread_rng();
    let max_moves = 200;

    let mut move_list = Vec::new();

    while board.status() == BoardStatus::Ongoing && move_list.len() < max_moves {
        let mv = pick_random_move(&board, &mut rng);
        move_list.push(mv.clone());
        board = board.make_move_new(mv)
    }

    println!("{}", pgn::basic_pgn(&move_list));
    print_end_of_game(&board);
}
