use chess;
use chess::{Board, BoardStatus, ChessMove};
use crate::eval;
use crate::eval::EvalFun;
use super::searchtree;
use super::ChessPlayer;
use std::time::{Duration, Instant};
use crate::utils::OrdByKey;
use crate::utils::display;
use crate::utils::display::JsonBuilder;
use std::mem;
use std::mem::MaybeUninit;
use crate::logging;
use crate::utils::fairheap::FairHeap;
use std::thread;
use std::sync;
use std::sync::{Arc, Mutex};
use crate::play;

/* A lock-based implementation of A-Star */
/* The locking is performed on the first level of the search tree, effectively
 * disallowing different threads from searching the same subtree.
 */

pub type ThreadCount = u8;

pub struct AStarPrl {
    time_budget: Duration,
    eval:        EvalFun,
    n_threads:   ThreadCount
}

impl ChessPlayer for AStarPrl {
    fn pick_move(&mut self, board: &Board, logger: &mut play::Logger) -> ChessMove {
        let init_tree = init_root(board.clone(), self.eval);
        let shared_tree = Arc::new(Mutex::new(init_tree));
        let eval_fun = self.eval;
        /*let shared_logger = Arc::new(Mutex::new(logger))*/

        let stop_time = Instant::now() + self.time_budget;
        let mut threads = Vec::new();
        for _ in 0..self.n_threads {
            let tree_ref = Arc::clone(&shared_tree);
            threads.push(
                thread::spawn(move || parallel_search(tree_ref, stop_time, eval_fun/*, shared_logger*/))
            )
        }
        //let search_tree = astar_search(board, self.eval, self.time_budget);
        for t in threads {
            t.join().unwrap()
        }

        let final_shared_tree = Arc::try_unwrap(shared_tree)
                            .unwrap_or_else(|arc| panic!("More than one ref remains: {} left",
                                                         Arc::strong_count(&arc)));
        let final_tree = final_shared_tree.into_inner().expect("Lock was poisoned");
        super::finalize(&final_tree, self.eval, self.time_budget, logger)
    }
}

type SeqTree = super::SearchTree;
type Shared<T> = Mutex<T>;
type SharedTree = Shared<SeqTree>;

//type PrlRoot = BinHeap<OrdByKey<Score, SeqBranch>>;
type PrlRoot = SeqTree;

fn init_root(init_board: Board, eval_fun: EvalFun) -> PrlRoot {
    super::init_root(init_board, eval_fun)
}

//fn lock_heap(shared_tree: &SharedTree) -> sync::LockResult<sync::MutexGuard<&mut super::NodeData>> {
    //shared_tree
        //.lock()
        //.map(|node| &mut node.node_data)
//}

//fn heap_op<F, T>(shared_tree: &SharedTree, f: F) -> Result<T, &'static str>
    //where F: FnMut<&mut super::NodeData> -> Result<T, &'static str>
//{
    //shared_tree
        //.lock()
        //.map_err(|_| "Lock is poisoned")
        //.and_then(f)
//}

//fn heap_op<F, T>(shared_tree: &SharedTree, f: F) -> LockResult<T>
    //where F: FnMut<&mut super::NodeData> -> T
//{
    //shared_tree
        //.lock()
        //.map(f)
        //.map_err(|_| Poisoned)
//}

struct Poisoned;
type LockResult<T> = Result<T, Poisoned>;

fn safe_get(shared_tree: &SharedTree) -> LockResult<Option<super::HeapEntry>> {
    //lock_heap(shared_tree)
        //.and_then(|heap| heap.pop())
    //shared_tree
        //.lock()
        //.map_err(|_| "Lock is poisoned")
        //.and_then(|node|
            //node.node_data.pop()
                //.ok_or("Heap is empty")
        //)
    //shared_tree
        //.lock()
        //.map_err(|_| Poisoned)
        //.map(|node| node.node_data.pop())
    //heap_op(shared_tree, |heap| heap.pop().ok_or("Heap is empty"))
    match shared_tree.lock() {
        Ok(mut node) => Ok(node.node_data.pop()),
        Err(_)   => Err(Poisoned)
    }
}

fn safe_set(shared_tree: &Mutex<SeqTree>, elem: super::HeapEntry) -> LockResult<()> {
    //lock_heap(shared_tree)
        //.and_then(|heap| heap.push(elem))
    //shared_tree
        //.lock()
        //.map_err(|_| Poisoned)
        //.map(|node| node.node_data.push(elem))
    match shared_tree.lock() {
        Ok(mut node) => Ok(node.node_data.push(elem)),
        Err(_)   => Err(Poisoned)
    }
}

fn parallel_search(
    shared_tree:   Arc<SharedTree>,
    stop_time:     Instant,
    eval_fun:      EvalFun/*,
    thread_logger: Arc<Mutex<&mut play::Logger>>*/)
{
    let root_board = shared_tree.lock().unwrap().board.clone();
    let root_player = root_board.side_to_move();
    while Instant::now() < stop_time {
        /* LOCK: BEGIN */
        let mut root = shared_tree.lock().unwrap();
        let mut heap = &mut root.node_data;
        let mv_idx = match heap.pop() {
            Some(entry) => entry.mv_idx,
            None => {
                /* This means there are too many threads compared to the number
                 * of possible moves at the root of the tree.
                 * Print a warning then stop the thread;
                 */
                 /* TODO */
                /*warn!(&mut thread_logger.lock().unwrap(),
                      "Thread {:?} stopped: not enough moves at the root level",
                      thread::current().id());*/
                return;  /* stop this thread */
            }
        };

        let branch_ptr = &mut root.moves[mv_idx] as *mut super::SearchMove;
        /* From this point on there should be automatic unlocking, as we're
         * not using anything from 'root'
         */
        /* LOCK: END */

        /* Access the branch in unsafe mode.
         * This is actually safe as the heap pop guarantees that the current
         * thread is the only one to have this mv_idx.
         */
        let branch: &mut _ = unsafe { &mut *branch_ptr };
        /* Perform the descent in lock-free mode, starting from the branch */
        let new_scores = super::continue_descent(branch, &root_board, eval_fun);

        /* Update the root */
        /* Need to lock again here */
        /* LOCK: BEGIN */
        let mut root = shared_tree.lock().unwrap();
        let mut heap = &mut root.node_data;
        let new_entry = super::HeapEntry {
            score: new_scores.get(root_player),
            mv_idx
        };
        heap.push(new_entry);
        /* LOCK: END */
    }
}

#[allow(dead_code)]
pub fn parallel_player(time_budget: Duration, n_threads: ThreadCount) -> impl ChessPlayer {
    AStarPrl {
        time_budget,
        eval: eval::classic_eval,
        n_threads
    }
}




/*
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
    let best_move_info = best_kv.0.value;
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
    let new_val_from_prev_best = OrdByKey::from(
        new_scores_from_prev_best[eval_player.to_index()],
        MoveAgg {
            scores: new_scores_from_prev_best,
            mv_idx:   best_move_idx
        }
    );
    ord_moves.push(new_val_from_prev_best);

    // FIXME handle finished games
    ord_moves.peek().unwrap().0.value.scores.clone()
}

fn expand(board: Board, eval_fun: EvalFun) -> SearchNode {
    let mut new_node = SearchNode::new(board, NodeData::default(), |_, _| ());
    new_node.node_data = base_search(&board, &new_node.moves, eval_fun);
    return new_node;
}

fn base_search(board: &Board, moves: &[SearchMove], eval_fun: EvalFun) -> NodeData {
    let mut ord_moves = MaxHeap::new();
    let eval_player = board.side_to_move();
    for mv_idx in 0..moves.len() {
        let mv = moves[mv_idx].mv;
        let res_board  = board.make_move_new(mv);
        let res_scores = both_scores(&res_board, eval_fun);
        ord_moves.push(OrdByKey::from(
            res_scores[eval_player.to_index()],
            MoveAgg {
                scores: res_scores,
                mv_idx: mv_idx
            }
        ));
    }
    return ord_moves;
}
*/
