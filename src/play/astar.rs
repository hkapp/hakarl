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

pub struct AStar {
    time_budget: Duration,
    eval:        EvalFun,
}

impl ChessPlayer for AStar {
    fn pick_move(&mut self, board: &Board, logger: &mut super::Logger) -> ChessMove {
        let search_tree = astar_search(board, self.eval, self.time_budget);

        print_tree_statistics(&search_tree, self.eval, self.time_budget, logger);

        return best_move(&search_tree);
    }

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

fn ordered_move_data(node: &SearchNode) -> Vec<OrdContent> {
    node.node_data.clone().into_sorted_vec()
}

fn ordered_branches(node: &SearchNode) -> Vec<&SearchMove> {
    ordered_move_data(node)
        .into_iter()
        .map(|mvdat| &node.moves[mvdat.0.value.mv_idx])
        .collect()
}

fn print_json_tree(tree: &SearchTree, eval_fun: EvalFun, logger: &mut super::Logger) {
    fn rec_build_json(node: &SearchNode, json: &mut JsonBuilder) {
        let sorted_moves = ordered_move_data(node);
        for mvdat in sorted_moves {
            let move_idx = mvdat.0.value.mv_idx;
            let move_branch = &node.moves[move_idx];

            let mv = move_branch.mv;
            let mv_str = format!("{}", mv);

            let mv_val = mvdat.0.key;
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
    while let Some(move_data) = curr_node.and_then(best_move_info) {
        //let move_data = best_move_info(curr_node);
        let move_idx = move_data.mv_idx;
        let branch = &curr_node.unwrap().moves[move_idx];
        line.push(branch.mv);
        curr_node = branch.child_node.as_ref();
    }
    return line;
}

type SearchTree = SearchNode;
type SearchNode = searchtree::Node<NodeData, MoveData>;
type SearchMove = searchtree::Branch<NodeData, MoveData>;

type MaxHeap<T>  = FairHeap<T>;
type NodeData    = MaxHeap<OrdContent>;
type OrdContent  = OrdByKey<eval::Score, MoveAgg>;

#[derive(Clone)]
struct MoveAgg {
    scores: BothScores,
    mv_idx: usize
}

type BothScores = [eval::Score; chess::NUM_COLORS];

type MoveData = ();

fn astar_search(
    board:       &Board,
    eval_fun:    EvalFun,
    time_budget: Duration)
    -> SearchTree
{
    let start_time = Instant::now();
    let mut tree = expand(board.clone(), eval_fun);

    while start_time.elapsed() < time_budget {
        descent(&mut tree, eval_fun);
    }

    return tree;
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
    best_kv.map(|kv| &kv.0.value)
}

#[allow(dead_code)]
pub fn astar_player(time_budget: Duration) -> impl ChessPlayer {
    AStar {
        time_budget,
        eval: eval::classic_eval,
    }
}
