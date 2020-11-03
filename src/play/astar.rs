use chess;
use chess::{Board, BoardStatus, ChessMove};
use crate::eval;
use crate::eval::EvalFun;
use super::searchtree;
use super::ChessPlayer;
use std::collections::BinaryHeap;
use std::time::{Duration, Instant};
use crate::utils::OrdBy;
use std::mem;
use std::mem::MaybeUninit;

pub struct AStar {
    time_budget: Duration,
    eval:        EvalFun,
}

impl ChessPlayer for AStar {
    fn pick_move(&mut self, board: &Board, _logger: &mut super::Logger) -> ChessMove {
        astar_search(board, self.eval, self.time_budget)
    }
}

type SearchTree = SearchNode;
type SearchNode = searchtree::Node<NodeData, MoveData>;
type SearchMove = searchtree::Branch<NodeData, MoveData>;

type NodeData    = BinaryHeap<OrdContent>;
type OrdContent  = OrdBy<eval::Score, MoveAgg>;

struct MoveAgg {
    scores: BothScores,
    mv_idx:   usize
}

type BothScores = [eval::Score; chess::NUM_COLORS];

type MoveData = ();

fn astar_search(
    board:    &Board,
    eval_fun: EvalFun,
    time_budget:    Duration)
    -> ChessMove
{
    let start_time = Instant::now();
    let mut tree = expand(board.clone(), eval_fun);

    while start_time.elapsed() < time_budget {
        descent(&mut tree, eval_fun);
    }

    best_move(&tree)
}

fn descent(search_node: &mut SearchNode, eval_fun: EvalFun) -> BothScores {
    // FIXME shortcut this code if the game is over
    let curr_board = &search_node.board;
    if curr_board.status() != BoardStatus::Ongoing {
        /* Game is over, return win / loss values */
        // Do we need to make sure that we don't hit this node again?
        return both_scores(curr_board, eval_fun);
    }

    let ord_moves      = &mut search_node.node_data;
    let best_kv        = ord_moves.pop().unwrap();
    let best_move_info = best_kv.data;
    let best_move_idx  = best_move_info.mv_idx;
    let best_branch    = &mut search_node.moves[best_move_idx];

    let new_scores_from_prev_best = match best_branch.child_node.as_mut() {
        Some(mut child) => descent(&mut child, eval_fun),
        None        => {
            /* expand this child */
            let mv = best_branch.mv;
            let new_board = curr_board.make_move_new(mv);
            let new_node  = expand(new_board, eval_fun);
            let scores    = best_scores(&new_node, eval_fun);
            best_branch.child_node = Some(new_node);
            scores
        }
    };

    let eval_player = curr_board.side_to_move();
    let new_val_from_prev_best = OrdBy {
        ord_key: new_scores_from_prev_best[eval_player.to_index()],
        data: MoveAgg {
            scores: new_scores_from_prev_best,
            mv_idx:   best_move_idx
        }
    };
    ord_moves.push(new_val_from_prev_best);

    // FIXME handle finished games
    ord_moves.peek().unwrap().data.scores.clone()
}

fn expand(board: Board, eval_fun: EvalFun) -> SearchNode {
    let mut new_node = SearchNode::new(board, NodeData::default(), |_, _| ());
    new_node.node_data = base_search(&board, &new_node.moves, eval_fun);
    return new_node;
}

fn base_search(board: &Board, moves: &[SearchMove], eval_fun: EvalFun) -> NodeData {
    let mut ord_moves = BinaryHeap::new();
    let eval_player = board.side_to_move();
    for mv_idx in 0..moves.len() {
        let mv = moves[mv_idx].mv;
        let res_board  = board.make_move_new(mv);
        let res_scores = both_scores(&res_board, eval_fun);
        ord_moves.push(OrdBy {
            ord_key: res_scores[eval_player.to_index()],
            data:    MoveAgg {
                scores: res_scores,
                mv_idx:   mv_idx
            }
        });
    }
    return ord_moves;
}

fn both_scores(board: &Board, eval: EvalFun) -> BothScores {
    let mut unsafe_scores: [MaybeUninit<eval::Score>; chess::NUM_COLORS] = unsafe {
        MaybeUninit::uninit().assume_init()
    };
    for player in &chess::ALL_COLORS {
        unsafe_scores[player.to_index()] = MaybeUninit::new(eval(board, *player));
    }
    return unsafe {
        mem::transmute(unsafe_scores)  // turn into safe array
    };
}

fn best_scores(node: &SearchNode, eval_fun: EvalFun) -> BothScores {
    match best_move_info(node) {
        Some(move_info) => move_info.scores.clone(),
        None            => both_scores(&node.board, eval_fun)
    }
}

fn best_move(tree: &SearchTree) -> ChessMove {
    let best_move_idx = best_move_info(tree).unwrap().mv_idx;
    tree.moves[best_move_idx].mv
}

fn best_move_info(node: &SearchTree) -> Option<&MoveAgg> {
    let ord_moves = &node.node_data;
    let best_kv = ord_moves.peek();
    best_kv.map(|kv| &kv.data)
}

const DEFAULT_TIME_BUDGET: Duration = Duration::from_millis(50);
pub fn astar_player() -> impl ChessPlayer {
    AStar {
        time_budget: DEFAULT_TIME_BUDGET,
        eval:        eval::classic_eval,
    }
}
