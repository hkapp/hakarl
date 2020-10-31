use chess::{Board, BoardStatus, ChessMove, MoveGen};
use rand::seq::IteratorRandom;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::utils;
use crate::eval;
use crate::eval::EvalFun;
use super::{MoveCount, ChessPlayer};

pub struct ExhaustiveSearch {
    depth: MoveCount,
    eval:  EvalFun,
    rng:   ThreadRng
}

impl ChessPlayer for ExhaustiveSearch {
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        println!();
        print!("{{start:{}, ", (self.eval)(board, board.side_to_move()));
        let (best_move, _best_board) = exhaustive_search(board,
                                                         self.eval,
                                                         self.depth,
                                                         &mut self.rng);
        println!("}}");
        println!("Best move: {}", best_move);
        return best_move;
    }
}

fn exhaustive_search<R: Rng>(
    board:    &Board,
    eval_fun: EvalFun,
    depth:    MoveCount,
    rng:      &mut R)
    -> (ChessMove, Board)
{
    let base_case = |mv| board.make_move_new(mv);

    let mut rec_case = |mv| {
        let next_board = board.make_move_new(mv);

        match next_board.status() {
            BoardStatus::Ongoing => {
                print!("{}:{{", mv);
                let r = exhaustive_search(&next_board, eval_fun, depth-1, rng).1;
                print!("}}, ");
                r
            }
            _                    => next_board  // stop recursion if game is over
        }
    };

    let mut board_for = |mv|
        if depth == 1 { base_case(mv) }
        else { rec_case(mv) };

    let player = board.side_to_move();

    let board_and_moves = MoveGen::new_legal(board).map(|mv| (mv, board_for(mv)));

    let best_moves = utils::iter::all_maxs_by_key(board_and_moves,
                                                  |(mv, new_board)| { let v = eval_fun(new_board, player); print!("{}:{}, ", mv, v); v });
    best_moves.into_iter()
              .choose(rng)
              .unwrap()
}

pub fn exhaustive_search_player(depth: MoveCount) -> impl ChessPlayer {
    ExhaustiveSearch {
        depth,
        eval: eval::classic_eval,
        rng:  rand::thread_rng()
    }
}
