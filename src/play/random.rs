use chess::{Board, MoveGen, ChessMove};
use rand::seq::IteratorRandom;
use rand::Rng;
use super::ChessPlayer;

fn pick_random_move<R: Rng>(board: &Board, rng: &mut R) -> ChessMove {
    let movegen = MoveGen::new_legal(&board);
    match movegen.choose(rng) {
        Some(mv) => mv,
        None     => panic!("Couldn't find a move!'")
    }
}

pub struct RandomPlayer<R: Rng> {
    rng: R
}

#[allow(dead_code)]
pub fn random_player() -> RandomPlayer<rand::rngs::ThreadRng> {
    RandomPlayer {
        rng: rand::thread_rng()
    }
}

impl<R: Rng> ChessPlayer for RandomPlayer<R> {
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        pick_random_move(board, &mut self.rng)
    }
}
