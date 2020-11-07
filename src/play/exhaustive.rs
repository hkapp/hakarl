use chess::{Board, BoardStatus, ChessMove, MoveGen};
use rand::seq::IteratorRandom;
use rand::Rng;
use rand::rngs::ThreadRng;
use crate::utils;
use crate::eval;
use crate::eval::EvalFun;
use super::{MoveCount, ChessPlayer};
use crate::logging::LogLevel;

pub struct ExhaustiveSearch {
    depth: MoveCount,
    eval:  EvalFun,
    rng:   ThreadRng
}

#[allow(unused_must_use)]
impl ChessPlayer for ExhaustiveSearch {
    fn pick_move(&mut self, board: &Board, logger: &mut super::Logger) -> ChessMove {
        let init_log_level = LogLevel::Debug;
        log_nol!(logger, init_log_level,
                 "\n{{start:{}, ", (self.eval)(board, board.side_to_move()));

        let (best_move, _best_board) = exhaustive_search(board,
                                                         self.eval,
                                                         self.depth,
                                                         &mut self.rng,
                                                         logger,
                                                         init_log_level);
        log!(logger, init_log_level, "}}");
        info!(logger, "Best move: {}", best_move);
        return best_move;
    }
}

#[allow(unused_must_use)]
fn exhaustive_search<R: Rng>(
    board:     &Board,
    eval_fun:  EvalFun,
    depth:     MoveCount,
    rng:       &mut R,
    logger:    &mut super::Logger,
    log_level: LogLevel)
    -> (ChessMove, Board)
{
    let base_case = |mv| board.make_move_new(mv);

    let mut rec_case = |mv| {
        let next_board = board.make_move_new(mv);

        match next_board.status() {
            BoardStatus::Ongoing => {
                log_nol!(logger, log_level, "{}:{{", mv);
                let r = exhaustive_search(&next_board, eval_fun, depth-1, rng, logger, log_level.lower()).1;
                log_nol!(logger, log_level, "}}, ");
                r
            }
            _                    => next_board  // stop recursion if game is over
        }
    };

    let mut board_for = |mv|
        if depth == 1 { base_case(mv) }
        else { rec_case(mv) };

    let player = board.side_to_move();

    /* Here we need to collect to avoid multiple mutable borrows of the logger by the closures */
    let board_and_moves: Vec<_> = MoveGen::new_legal(board).map(|mv| (mv, board_for(mv))).collect();

    let best_moves = utils::iter::all_maxs_by_key(board_and_moves.into_iter(),
                                                  |(mv, new_board)| { let v = eval_fun(new_board, player); log_nol!(logger, log_level, "{}:{}, ", mv, v); v });
    best_moves.into_iter()
              .choose(rng)
              .unwrap()
}

#[allow(dead_code)]
pub fn exhaustive_search_player(depth: MoveCount) -> impl ChessPlayer {
    ExhaustiveSearch {
        depth,
        eval: eval::classic_eval,
        rng:  rand::thread_rng()
    }
}
