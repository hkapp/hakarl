use chess::{Board, MoveGen, ChessMove};
use crate::eval::{EvalFun, Score};
use crate::eval;
use super::ChessPlayer;
use rand::seq::IteratorRandom;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::utils;

fn pick_best_move<R: Rng>(board: &Board, eval: EvalFun, rng: &mut R) -> ChessMove {
    let curr_player = board.side_to_move();

    /* this is a closure */
    let eval_move = |mv: &ChessMove| -> Score {
        let state_after_move = board.make_move_new(mv.clone());
        eval(&state_after_move, curr_player)
    };

    let movegen = MoveGen::new_legal(&board);
    let best_moves = utils::iter::all_maxs_by_key(movegen, eval_move);
    best_moves.into_iter().choose(rng).unwrap()
}

#[derive(Clone)]
pub struct EvalPlayer {
    eval: EvalFun,
    rng:  ThreadRng
}

pub fn eval_driven_player(eval: EvalFun) -> EvalPlayer {
    EvalPlayer {
        eval,
        rng: rand::thread_rng()
    }
}

#[allow(dead_code)]
pub fn classic_eval_player() -> EvalPlayer {
    eval_driven_player(eval::classic_eval)
}

impl ChessPlayer for EvalPlayer {
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        pick_best_move(board, self.eval, &mut self.rng)
    }
}
