mod asprl;

use chess;
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
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
use std::cmp;

pub struct AStar {
    time_budget: Duration,
    eval:        EvalFun,
}

impl ChessPlayer for AStar {
    fn pick_move(&mut self, board: &Board, logger: &mut super::Logger) -> ChessMove {
        let search_tree = astar_search(board, self.eval, self.time_budget);

        finalize(&search_tree, self.eval, self.time_budget, logger)
    }

}

fn finalize(
    final_tree: &SearchTree,
    eval_fun:   EvalFun,
    run_dur:    Duration,
    logger:     &mut super::Logger)
    -> ChessMove
{
    print_tree_statistics(&final_tree, eval_fun, run_dur, logger);

    return best_move(&final_tree).unwrap();
}

fn print_tree_statistics(
    tree:     &SearchTree,
    eval_fun: EvalFun,
    duration: Duration,
    logger:   &mut super::Logger)
{
    let node_count = tree.count_nodes();
    let ms = duration.as_millis();

    info!(logger, "[AStar statistics]");
    info!(logger, "  {} nodes searched in {}ms", node_count, ms);
    info!(logger, "  average: {:.1} nodes per second", 1000. * (node_count as f32 / ms as f32));
    info!(logger, "  max tree depth: {}", tree.depth());

    let level1_depths: Vec<_> = tree.moves.iter()
                                          .map(|branch| match &branch.child_node {
                                              Some(child) => child.depth() + 1,
                                              None        => 1,
                                          })
                                          .collect();
    info!(logger, "  level-1 tree depth: max={}, min={}",
                  level1_depths.iter().max().unwrap(),
                  level1_depths.iter().min().unwrap());

    print_best_lines(tree, logger);

    if logger.allows(logging::LogLevel::Trace) {
        print_json_tree(tree, eval_fun, logger);
    }
}

fn ordered_move_data(node: &SearchNode) -> Vec<HeapEntry> {
    node.node_data.clone().into_sorted_vec()
}

fn ordered_branches(node: &SearchNode) -> Vec<&SearchMove> {
    ordered_move_data(node)
        .into_iter()
        .map(|entry| &node.moves[entry.mv_idx])
        .collect()
}

fn print_json_tree(tree: &SearchTree, eval_fun: EvalFun, logger: &mut super::Logger) {
    fn rec_build_json(node: &SearchNode, json: &mut JsonBuilder) {
        let sorted_moves = ordered_move_data(node);
        for mvdat in sorted_moves {
            let move_idx = mvdat.mv_idx;
            let move_branch = &node.moves[move_idx];

            let mv = move_branch.mv;
            let mv_str = format!("{}", mv);

            let mv_val = mvdat.score;
            let val_str = format!("{}", mv_val);

            match move_branch.child_node.as_ref() {
                Some(child) => {
                    json.open_rec(mv_str);
                    json.push(String::from("value"), val_str);
                    rec_build_json(&child, json);
                    json.close_rec();
                }

                None => {
                    json.push(mv_str, val_str);
                }
            }
        }
    }

    let mut json = JsonBuilder::new();

    let init_board = &tree.board;
    let eval_player = init_board.side_to_move();
    let board_val = eval_fun(init_board, eval_player);
    let init_val_str = format!("{}", board_val);
    json.push(String::from("init_value"), init_val_str);

    rec_build_json(tree, &mut json);

    match json.to_string() {
        Ok(json_str)  => trace!(logger, "{}", json_str),
        Err(json_err) => warn!(logger, "{}", json_err),
    };
}

fn print_best_lines(tree: &SearchTree, logger: &mut super::Logger) {
    fn format_line(line: &[ChessMove]) -> String {
        let formatted_moves = line.iter().map(|mv| format!("{}", mv));
        display::join(formatted_moves, " -> ")
    }

    #[allow(unused_must_use)]
    fn print_line_starting(branch: &SearchMove, line_prefix: &str, writer: &mut dyn std::io::Write) {
        let mut line = match &branch.child_node {
            Some(node) => best_line(&node),
            None       => Vec::new()
        };
        line.insert(0, branch.mv);
        writeln!(writer, "{}{}", line_prefix, format_line(&line));
    }

    let ordered_branches = ordered_branches(tree);
    assert!(ordered_branches.len() >= 1);

    use logging::LogLevel;
    logger.writer(LogLevel::Info)
        .map(|writer| print_line_starting(&ordered_branches[0], "  Best line: ", writer));

    debug!(logger, "  Other lines (ordered):");
    logger.writer(LogLevel::Debug)
        .map(|writer|
            for i in 1..ordered_branches.len() {
                print_line_starting(&ordered_branches[i], "    ", writer)
            });
}

fn best_line(tree: &SearchTree) -> Vec<ChessMove> {
    let mut curr_node = Some(tree);
    let mut line = Vec::new();
    while let Some(branch) = curr_node.and_then(best_branch) {
        //let move_data = best_move_info(curr_node);
        //let move_idx = move_data.mv_idx;
        //let branch = &curr_node.unwrap().moves[move_idx];
        line.push(branch.mv);
        curr_node = branch.child_node.as_ref();
    }
    return line;
}

type SearchTree = SearchNode;
type SearchNode = searchtree::Node<NodeData, MoveData>;
type SearchMove = searchtree::Branch<NodeData, MoveData>;

type MaxHeap<T>  = FairHeap<T>;
type NodeData    = MaxHeap<HeapEntry>;

#[derive(Eq, Clone, Copy)]
struct HeapEntry {
    score:  eval::Score,
    mv_idx: usize
}

/* We sort by score in the heap */
impl PartialOrd for HeapEntry {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        self.score.partial_cmp(&other.score)
    }
}

impl PartialEq for HeapEntry {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Ord for HeapEntry {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.score.cmp(&other.score)
    }
}

type MoveData   = BothScores;

//fn both_scores(board: &Board, eval: EvalFun) -> BothScores {
    //let mut unsafe_scores: [MaybeUninit<eval::Score>; chess::NUM_COLORS] = unsafe {
        //MaybeUninit::uninit().assume_init()
    //};
    //for player in &chess::ALL_COLORS {
        //unsafe_scores[player.to_index()] = MaybeUninit::new(eval(board, *player));
    //}
    //return unsafe {
        //mem::transmute(unsafe_scores)  // turn into safe array
    //};
//}

#[derive(Clone, Copy)]
//struct BothScores([eval::Score; chess::NUM_COLORS]);
struct BothScores {
    white_score: eval::Score,
    black_score: eval::Score,
}
impl BothScores {
    fn new(white_score: eval::Score, black_score: eval::Score) -> Self {
        BothScores {
            white_score,
            black_score
        }
        //assert!(Color::White.as_index() == 0 && )
        //let mut unsafe_scores: [MaybeUninit<eval::Score>; chess::NUM_COLORS] = unsafe {
            //MaybeUninit::uninit().assume_init()
        //};
        //let white_idx = Color::White.to_index();
        //unsafe_scores[white_idx] = MaybeUninit::new(white_score);
        //let black_idx = Color::Black.to_index();
        //unsafe_scores[white_idx] = MaybeUninit::new(white_score);
        //return unsafe {
            //mem::transmute(unsafe_scores)  // turn into safe array
        //};
    }

    fn get(&self, player: Color) -> eval::Score {
        //self.0[player.as_index()]
        match player {
            Color::White => self.white_score,
            Color::Black => self.black_score,
        }
    }

    fn build_from(board: &Board, eval: EvalFun) -> Self {
        Self::new(
            eval(board, Color::White),
            eval(board, Color::Black)
        )
    }
}

//#[derive(Clone)]
//struct MoveAgg {
    //scores: BothScores,
    //mv_idx: usize
//}

fn astar_search(
    board:       &Board,
    eval_fun:    EvalFun,
    time_budget: Duration)
    -> SearchTree
{
    let start_time = Instant::now();
    let mut tree = init_root(board.clone(), eval_fun);

    while start_time.elapsed() < time_budget {
        descent(&mut tree, eval_fun);
    }

    return tree;
}

fn init_root(init_board: Board, eval_fun: EvalFun) -> SearchTree {
    new_node(init_board, eval_fun)
}

fn descent(node: &mut SearchNode, eval_fun: EvalFun) -> BothScores {
    // FIXME shortcut this code if the game is over
    let curr_board = &node.board;
    if curr_board.status() != BoardStatus::Ongoing {
        /* Game is over, return win / loss values */
        // Do we need to make sure that we don't hit this node again?
        return BothScores::build_from(curr_board, eval_fun);
    }

    let heap       = &mut node.node_data;
    let best_entry = heap.pop().unwrap();
    let mv_idx     = best_entry.mv_idx;
    let branch     = &mut node.moves[mv_idx];

    let new_scores = continue_descent(branch, curr_board, eval_fun);

    /* Update the heap */
    let eval_player = curr_board.side_to_move();
    let new_heap_entry = HeapEntry {
        score:  new_scores.get(eval_player),
        mv_idx
    };
    heap.push(new_heap_entry);

    // FIXME handle finished games
    return new_scores;
}

fn continue_descent(branch: &mut SearchMove, prev_board: &Board, eval_fun: EvalFun) -> BothScores {
    let new_scores = match branch.child_node.as_mut() {
        Some(mut child) => {
            /* child node already expanded: recursively descent */
            descent(&mut child, eval_fun)
        },
        None => {
            /* child not exanded yet: do it now and stop the recursion */
            expand(branch, prev_board, eval_fun);
            best_scores(branch.child_node.as_ref().unwrap(), eval_fun)
        }
    };

    /* Update the branch data */
    branch.mv_data = new_scores;

    return new_scores;
}

fn expand(branch: &mut SearchMove, prev_board: &Board, eval_fun: EvalFun) {
    let mv        = branch.mv;
    let new_board = prev_board.make_move_new(mv);
    let new_child = new_node(new_board, eval_fun);

    /* Mark this branch as expanded by storing the child node */
    branch.child_node = Some(new_child);
}

fn new_node(board: Board, eval_fun: EvalFun) -> SearchNode {
    /* Step 1: create the branches, with evaluation */
    fn create_branches(board: &Board, eval_fun: EvalFun) -> Vec<SearchMove> {
        let mut branches = Vec::new();
        for mv in MoveGen::new_legal(board) {
            branches.push(
                SearchMove {
                    mv,
                    mv_data: BothScores::build_from(board, eval_fun),
                    child_node: None
                }
            )
        }
        return branches;
    }

    /* Step 2: Build the initial heap state */
    fn build_heap(moves: &[SearchMove], player: Color) -> NodeData {
        let mut heap = MaxHeap::new();
        for mv_idx in 0..moves.len() {
            let branch = &moves[mv_idx];
            let scores = branch.mv_data;
            let new_entry = HeapEntry {
                score: scores.get(player),
                mv_idx
            };
            heap.push(new_entry);
        }
        return heap;
    }

    let branches = create_branches(&board, eval_fun);
    let eval_player = board.side_to_move();

    SearchNode {
        board,
        node_data: build_heap(&branches, eval_player),
        moves: branches
    }

    /*let mut node = SearchNode::new(board, NodeData::default(), |_, _| ());
    node.node_data = base_search(&board, &mv_data.moves, eval_fun);
    return node;*/
}
/*
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
fn best_scores(node: &SearchNode, eval_fun: EvalFun) -> BothScores {
    match best_branch(node) {
        Some(branch) => branch.mv_data /*scores*/,
        None         => BothScores::build_from(&node.board, eval_fun)
    }
}

fn best_move(node: &SearchNode) -> Option<ChessMove> {
    best_branch(node)
        .map(|b| b.mv)
}

fn best_branch(node: &SearchNode) -> Option<&SearchMove> {
    best_mv_idx(node)
        .map(|mv_idx| &node.moves[mv_idx])
}

fn best_mv_idx(node: &SearchNode) -> Option<usize> {
    let heap = &node.node_data;
    let best_entry = heap.peek();
    best_entry.map(|e| e.mv_idx)
}

#[allow(dead_code)]
pub fn astar_player(time_budget: Duration) -> impl ChessPlayer {
    AStar {
        time_budget,
        eval: eval::classic_eval,
    }
}
