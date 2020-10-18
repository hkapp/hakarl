use chess::{Board, MoveGen, ChessMove};
use crate::eval::{EvalFun, Score};
use crate::eval;
use super::ChessPlayer;
use rand::seq::IteratorRandom;
use rand::Rng;
use rand::rngs::ThreadRng;

fn all_maxs_by_key<B, F, I>(mut iter: I, mut f: F) -> Vec<I::Item>
    where
        I: Iterator,
        F: FnMut(&I::Item) -> B,
        B: Ord
{
    let first_item = iter.next();

    if !first_item.is_some() {
        return Vec::new();
    }

    /* We are now sure to have an initial value */
    let first_item = first_item.unwrap();
    let mut max_val = f(&first_item);
    let mut max_items = vec![first_item];

    for item in iter {
        let item_val = f(&item);

        if item_val > max_val {
            max_val = item_val;
            max_items.clear();
            max_items.push(item);
        }
        else if item_val == max_val {
            max_items.push(item);
        }
    }

    return max_items;
}

fn pick_best_move<R: Rng>(board: &Board, eval: EvalFun, rng: &mut R) -> ChessMove {
    let curr_player = board.side_to_move();

    /* this is a closure */
    let eval_move = |mv: &ChessMove| -> Score {
        let state_after_move = board.make_move_new(mv.clone());
        eval(&state_after_move, curr_player)
    };

    let movegen = MoveGen::new_legal(&board);
    //movegen.max_by_key(eval_move).unwrap()
    let best_moves = all_maxs_by_key(movegen, eval_move);
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

pub fn classic_eval_player() -> EvalPlayer {
    eval_driven_player(eval::classic_eval)
}

impl ChessPlayer for EvalPlayer {
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        pick_best_move(board, self.eval, &mut self.rng)
    }
}
