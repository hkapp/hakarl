use chess;
use chess::{Board, BoardStatus, ChessMove, Color, MoveGen};
use crate::eval;
use crate::eval::EvalFun;
use super::searchtree;
use super::{ChessPlayer, DebugPlayer};
use std::time::{Duration, Instant};
use crate::utils::display;
use crate::utils::display::JsonBuilder;
use crate::logging;
use crate::play;
use crate::utils::fairheap::FairHeap;
use std::cmp;
use crate::utils::{Either, Either::Left, Either::Right};
use crate::utils::dot;
use std::fmt;
use super::BothScores;
use std::sync::atomic;
use std::sync::atomic::AtomicU32;
use atomic_option::AtomicOption;

/* LFStar ChessPlayer */

pub struct LFStar {
    time_budget: Duration,
    eval:        EvalFun,
}

type Logger = play::Logger;
pub type OpaqueTree = LFTree;  /* remove if not necessary */

impl DebugPlayer for LFStar {
    type DebugData = LFTree;

    fn compute_move(&mut self, board: &Board, logger: &mut Logger) -> Self::DebugData {
        let search_tree = astar_search(board, self.eval, self.time_budget);

        print_tree_statistics(&search_tree, self.eval, self.time_budget, logger);

        // TODO make this recursive to check the entire tree
        // TODO uncomment
        /*debug_assert!(node_is_consistent(search_tree), "Inconsistent node after regular descent");*/

        return search_tree;
    }

    fn best_move(&self, tree: &Self::DebugData) -> ChessMove {
        best_move(&tree).unwrap()
    }
}

const DEFAULT_EVAL_FUN: EvalFun = eval::classic_eval;
#[allow(dead_code)]
pub fn astar_player(time_budget: Duration) -> LFStar {
    LFStar {
        time_budget,
        eval: DEFAULT_EVAL_FUN,
    }
}

/* Data structures used for the search */

type LFTree = LFNode;

//type LFNode = searchtree::Node<NodeData, MoveData>;
struct LFNode {
    board: Board,
    moves: Vec<LFBranch>
}

//type LFBranch = searchtree::Branch<NodeData, MoveData>;
struct LFBranch {
    mv:          ChessMove,
    scores:      AtomicScores,
    child_node:  AtomicChild
}

/* LFNode and LFBranch APIs */

impl LFNode {
    fn is_final(&self) -> bool {
        self.board.status() != BoardStatus::Ongoing
    }

    fn current_player(&self) -> Color {
        self.board.side_to_move()
    }
}

impl LFBranch {
    /* Should we mark this function as unsafe? */
    fn score_of(&self, player: Color) -> BothScores {
        self.scores.load().get(player)
    }
}

/* packed pairs */

fn pack_pair_i16(a: i16, b: i16) -> u32 {
    let a_bytes:  [u8; 2] = a.to_ne_bytes();
    let b_bytes:  [u8; 2] = b.to_ne_bytes();
    let ab_bytes: [u8; 4] = [a_bytes[0], a_bytes[1], b_bytes[0], b_bytes[1]];

    u32::from_ne_bytes(ab_bytes)
}

fn unpack_pair_i16(packed: u32) -> (i16, i16) {
    let packed_bytes: [u8; 4] = packed.to_ne_bytes();
    let a_bytes:      [u8; 2] = [packed_bytes[0], packed_bytes[1]];
    let b_bytes:      [u8; 2] = [packed_bytes[2], packed_bytes[3]];

    let a = i16::from_ne_bytes(a_bytes);
    let b = i16::from_ne_bytes(b_bytes);
    (a, b)
}

/* packed scores */

fn pack_scores(scores: BothScores) -> u32 {
    pack_pair_i16(scores.white_score, scores.black_score)
}

fn unpack_scores(packed_scores: u32) -> BothScores {
    let (white_score, black_score) = unpack_pair_i16(packed_scores);

    BothScores {
        white_score,
        black_score
    }
}

/* AtomicScores */
struct AtomicScores(AtomicU32);

impl AtomicScores {
    const LOAD_ORDERING: atomic::Ordering = atomic::Ordering::Relaxed;
    const CAS_ORDERING:  atomic::Ordering = atomic::Ordering::Relaxed;

    /* we might be able to improve the performance of the pack/unpack code
     * by using direct pointer casting.
     */
    /* should be able to use transmute with repr(transparent) */
    const fn new(scores: BothScores) -> Self {
        let packed_scores = pack_scores(scores);

        AtomicScores(
            AtomicU32::new(packed_scores))
    }

    fn load(&self) -> BothScores {
        let packed_scores = self.0.load(Self::LOAD_ORDERING);

        unpack_scores(packed_scores)
    }

    fn compare_and_swap(&self, current: BothScores, new: BothScores) -> BothScores
    {
        let packed_current = pack_scores(current);
        let packed_new = pack_scores(new);

        let raw_cas_value = self.0.compare_and_swap(packed_current, packed_new, Self::CAS_ORDERING);

        unpack_scores(raw_cas_value)
    }
}

/* AtomicChild */
struct AtomicChild(AtomicOption<LFNode>);

impl AtomicChild {
    const LOAD_ORDERING: atomic::Ordering = atomic::Ordering::Relaxed;

    fn empty() -> Self {
        AtomicChild(
            AtomicOption::empty())
    }

    fn load_ref(&self) -> Option<&LFNode> {
        let raw_ptr = self.0.load_raw(Self::LOAD_ORDERING);
        if raw_ptr.is_null() {
            None
        }
        else {
            Some(
                unsafe { &*raw_ptr } /* FIXME this cast seems wrong */
            )
        }
    }

    fn try_store(&self, child: LFNode) {
        self.0.try_store(Box::new(child));
    }
}

/********** LFStar search code **********/

fn astar_search(
    board:       &Board,
    eval_fun:    EvalFun,
    time_budget: Duration)
    -> LFTree
{
    let start_time = Instant::now();
    let mut tree = init_root(board.clone(), eval_fun);

    while start_time.elapsed() < time_budget {
        descent(&mut tree, eval_fun);
    }

    return tree;
}

fn init_root(init_board: Board, eval_fun: EvalFun) -> LFTree {
    new_node(init_board, eval_fun)
}

fn descent(node: &LFNode, eval_fun: EvalFun) {
    // FIXME shortcut this code if the game is over
    if node.is_final() {
        /* Game is over, return win / loss values */
        // Do we need to make sure that we don't hit this node again?
        return;
    }

    /* Getting the best branch is unsafe here.
     * That is, the resulting branch we get might not be the best as soon as we receive it.
     * This is fine though, we'll just investigate a less-than optimal branch.
     */
    let branch = unsafe { best_branch(node) };

    continue_descent(branch.unwrap(), &node.board, eval_fun);

    // FIXME handle finished games
}

fn continue_descent(branch: &LFBranch, prev_board: &Board, eval_fun: EvalFun) {
    let child_node = match branch.child_node.load_ref() {
        Some(child) => {
            /* child node already expanded: recursively descent */
            descent(&child, eval_fun);
            &child  /* pass up the existing child */
        },
        None => {
            /* child not exanded yet: do it now and stop the recursion */
            expand(branch, prev_board, eval_fun);
            branch.child_node.load_ref().unwrap() /* pass up the new child */
        }
    };

    update_branch(branch, child_node, eval_fun);
}

fn update_branch(parent_branch: &LFBranch, child_node: &LFNode, eval_fun: EvalFun) {
    let cas_update = || {
        let prev_scores = parent_branch.scores.load();

        /* Compute the best scores now.
         * This is unsafe because the result might be outdated when we get it.
         * The CAS will decide on that.
         */
        let new_scores = unsafe { best_scores(child_node, eval_fun) };

        let cas_result = parent_branch.scores.compare_and_swap(prev_scores, new_scores);

        cas_result != prev_scores
    };

    /* If the CAS fails, we recompute the new best.
     * If another thread CASed before us, but with the same best value, we're happy
     * and can move on.
     */
    while !cas_update() {
        /* TODO we might want to log something here in case the CAS fails,
         *      just to give us an idea of how much conflicts there are.
         */
    }

    // TODO find a way to do consistency assertions
    /*debug_assert!(branch_is_consistent(branch, prev_board, eval_fun),
                  "Inconsistent branch after continue_descent");*/
}

fn expand(branch: &LFBranch, prev_board: &Board, eval_fun: EvalFun) {
    let mv        = branch.mv;
    let new_board = prev_board.make_move_new(mv);
    let new_child = new_node(new_board, eval_fun);

    // TOOD find a way to do consistency assertions
    /*debug_assert!(node_is_consistent(&new_child), "New expanded node is inconsistent");*/

    /* Try to expand the branch.
     * If another thread did it before this one, it's fine.
     */
    branch.child_node.try_store(new_child);
    // TODO just log something if this fails
    //      this would give an idea of the conflict ratio
}

fn new_node(board: Board, eval_fun: EvalFun) -> LFNode {
    /* Create the branches, with evaluation */
    fn create_branches(board: &Board, eval_fun: EvalFun) -> Vec<LFBranch> {
        let mut branches = Vec::new();
        for mv in MoveGen::new_legal(board) {
            /* Bug: Make sure to compute the scores wrt. the
             *      new board state.
             */
            let next_board = board.make_move_new(mv);
            let next_scores = BothScores::build_from(&next_board, eval_fun);
            branches.push(
                LFBranch {
                    mv,
                    scores: AtomicScores::new(next_scores),
                    child_node: AtomicChild::empty()
                }
            )
        }
        return branches;
    }

    let branches = create_branches(&board, eval_fun);
    /* This assertion is safe because we haven't shared the child yet */
    // TODO uncomment
    /*for b in branches.iter() {
        debug_assert!(branch_is_consistent(b, &board, eval_fun),
                      "Created an inconsistent branch in 'new_node()'");
    }*/

    LFNode {
        board,
        moves: branches
    }
}

/* Unsafe utilities */
/* All of these functions are unsafe because they require the node or branch
 * passed as argument to be "the only reference".
 * If other threads are still potentially working on this branch/node, the
 * results returned by these functions may be inconsistent.
 */

unsafe fn best_scores(node: &LFNode, eval_fun: EvalFun) -> BothScores {
    match best_branch(node) {
        Some(branch) => branch.scores.load(),
        None         => BothScores::build_from(&node.board, eval_fun)
    }
}

unsafe fn best_move(node: &LFNode) -> Option<ChessMove> {
    best_branch(node)
        .map(|b| b.mv)
}

unsafe fn best_branch(node: &LFNode) -> Option<&LFBranch> {
    let curr_player = node.current_player();
    node.moves.iter()
        .max_by_key(|branch| {
            let both_scores = branch.scores.load();
            both_scores.get(curr_player)
        })
}

unsafe fn sorted_branches(node: &LFNode) -> Vec<&LFBranch> {
    let mut sorted_refs: Vec<_> = node.moves.iter().collect();
    let curr_player = node.current_player();

    sorted_refs.sort_by_key(|b| b.score_of(curr_player));

    return sorted_refs;
}

/********** Consistency checks **********/
/* TODO reimplement those */

/*fn node_is_consistent(node: &LFNode) -> bool {
    let heap = &node.node_data;
    if node.board.status() != BoardStatus::Ongoing {
        if !heap.is_empty() {
            println!("Node's game is over, but its heap is not empty");
            return false;
        }
        if !node.moves.is_empty() {
            println!("Node's game is over, but it somehow contain moves");
            return false;
        }
        return true;
    }

    let node_player = node.board.side_to_move();
    let max_heap_entry = heap.peek().unwrap();

    /* 1. check that the max value in the heap is the same as the
     * max movedata.
     */
    let max_heap_val = max_heap_entry.score;

    let max_branch_val = node.moves.iter()
                            .map(|b| b.mv_data.get(node_player))
                            .max()
                            .unwrap();

    if max_heap_val != max_branch_val {
        println!("Inconsistent max values in node");
        println!("  max value by peeking into the heap: {}", max_heap_val);
        println!("  max value by iterating over the branches: {}", max_branch_val);
        return false;
    }

    /* 2. Check that the value of the max move in the heap
     * is the same as in the corresponding branch.
     */
    let best_heap_move = best_move(node).unwrap();
    let corr_branch = node.moves.iter()
                        .find(|b| b.mv == best_heap_move)
                        .unwrap();
    let branch_val = corr_branch.mv_data.get(node_player);

    if branch_val != max_heap_val {
        println!("The branch corresponding to the best move doesn't have the same value in the heap");
        println!("(computed using 'find()' on the branch list)");
        println!("  max value from the heap: {}", max_heap_val);
        println!("  branch value for the max move in the heap: {}", branch_val);
        return false;
    }

    /* 2b. Same check as 2., but done via best_branch
     */
    let best_branch = best_branch(node).unwrap();
    let branch_val = best_branch.mv_data.get(node_player);

    if branch_val != max_heap_val {
        println!("The branch corresponding to the best move doesn't have the same value in the heap");
        println!("(computed using 'best_branch()')");
        println!("  max value from the heap: {}", max_heap_val);
        println!("  branch value for the max move in the heap: {}", branch_val);
        return false;
    }

    return true;
}

fn branch_is_consistent(branch: &LFBranch, prev_board: &Board, eval_fun: EvalFun) -> bool {
    let print_additional_data = || {
        println!("");
        match branch.child_node.as_ref() {
            Some(child_node) => {
                println!("Parent is {:?}, child is {:?}", prev_board.side_to_move(), child_node.board.side_to_move());
                for child_entry in sorted_heap_entries(child_node) {
                    println!("HeapEntry(score: {}, mv_idx ~> {})",
                             child_entry.score,
                             child_node.moves[child_entry.mv_idx].mv_data);
                }
            }
            None => {
                /* nothing for the moment */
            }
        }
    };

    let prev_player = prev_board.side_to_move();
    let branch_val = branch.mv_data.get(prev_player);

    /* 1. For non-expanded branches, the branch value must be the
     *    value of the board.
     */
    if branch.child_node.is_none() {
        let next_board = prev_board.make_move_new(branch.mv);
        let next_board_val = eval_fun(&next_board, prev_player);

        if branch_val != next_board_val {
            println!("Simple board evaluation does not match branch data");
            println!("(encountered in unexpanded branch)");
            println!("  value from branch data: {}", branch_val);
            println!("  value from board evaluation: {}", next_board_val);
            print_additional_data();
            return false;
        }
    }
    else {
        let child_node = branch.child_node.as_ref().unwrap();
        match best_branch(child_node) {
            Some(best_child_branch) => {
                /* 2. If branch is expanded, branch data must be consistent with
                 *    best val in child heap.
                 */
                let best_child_val = best_child_branch.mv_data.get(prev_player);
                if best_child_val != branch_val {
                    println!("Branch value is inconsistent with best value from child");
                    println!("  value in branch data: {}", branch_val);
                    println!("  value of best branch in child: {}", best_child_val);
                    print_additional_data();
                    return false;
                }
             }
             None => {
                /* 3. Check values if the child board is over (finished game) */
             }
        }
    }

    return true;
}*/

/********** Debugging at the end of the search **********/

fn finalize(
    final_tree: &LFTree,
    eval_fun:   EvalFun,
    run_dur:    Duration,
    logger:     &mut Logger)
    -> ChessMove
{
    print_tree_statistics(&final_tree, eval_fun, run_dur, logger);

    return best_move(&final_tree).unwrap();
}

fn print_tree_statistics(
    tree:     &LFTree,
    eval_fun: EvalFun,
    duration: Duration,
    logger:   &mut Logger)
{
    let node_count = tree.count_nodes();
    let ms = duration.as_millis();

    info!(logger, "[LFStar statistics]");
    info!(logger, "  {} nodes searched in {}ms", node_count, ms);
    info!(logger, "  average: {:.1} nodes per second", 1000. * (node_count as f32 / ms as f32));
    info!(logger, "  max tree depth: {}", tree.depth());

    let level1_depths: Vec<_> = tree.moves.iter()
                                          .map(|branch| match &branch.child_node.load_ref() {
                                              Some(child) => child.depth() + 1,
                                              None        => 1,
                                          })
                                          .collect();
    info!(logger, "  level-1 tree depth: max={}, min={}",
                  level1_depths.iter().max().unwrap(),
                  level1_depths.iter().min().unwrap());

    print_best_lines(tree, eval_fun, logger);
}

/* This function is unsafe because there must be guarantees that no other thread is still
 * working on the tree passed as argument.
 */
unsafe fn print_best_lines(tree: &LFTree, eval_fun: EvalFun, logger: &mut Logger) {
    fn format_line(line: &[Either<&LFNode, &LFBranch>], eval_fun: EvalFun) -> String {
        //let formatted_moves = line.iter().map(|mv| format!("{}", mv));
        //display::join(formatted_moves, " -> ")
        /* The first element should always be the initial board */
        let init_board = &line[0].unwrap_left().board;
        let eval_player = init_board.side_to_move();

        let mut scored_line = line.iter().map(
            |elem| match elem {
                Left(node)    => Left(eval_fun(&node.board, eval_player)),
                Right(branch) => Right(branch.mv),
            }
        );

        let mut piecewise_format = Vec::new();
        /* the first element in the line is a board, so get its value */
        let init_value = scored_line.next().unwrap().unwrap_left();
        while let Some(mv_elem) = scored_line.next() {
            /* this has to be a move */
            let mv = mv_elem.unwrap_right();

            /* the next elem must be a board value */
            /* There may be no board if this is the end of the line
             * (the node hasn't been expanded)
             */
            let board_value = match scored_line.next() {
                Some(value_elem) => value_elem.unwrap_left(),
                None => {
                    /* This is the end of the line, so the node hasn't been expanded.
                     * Fetch the last board and evaluate it.
                     */
                    let all_boards = line.iter().filter_map(
                        |elem| match elem {
                            Left(node) => Some(&node.board),
                            Right(_)   => None
                        }
                    );
                    let last_board = all_boards.last().unwrap();
                    let final_value = eval_fun(last_board, eval_player);

                    final_value
                }
            };
            /* Print only the relative value of the move wrt. the initial state */
            let relative_value = board_value - init_value;

            piecewise_format.push(format!("{}({:+})", mv, relative_value));
        }

        return display::join(piecewise_format.into_iter(), " -> ");
    }

    #[allow(unused_must_use)]
    fn print_line_starting(
        tree:        &LFTree,
        branch:      &LFBranch,
        line_prefix: &str,
        eval_fun:    EvalFun,
        writer:      &mut dyn std::io::Write)
    {
        /* Make the line start at the node pointed to by mv_idx */
        let mut line = match &branch.child_node {
            Some(node) => best_line_full(&node),
            None       => Vec::new()
        };
        /* Add the move and board state that were skipped */
        line.insert(0, Left(tree));
        line.insert(1, Right(branch));

        /* Format and print the line */
        let formatted_line = format_line(&line, eval_fun);
        writeln!(writer, "{}{}", line_prefix, formatted_line);
    }

    let sorted_branches: Vec<_> = unsafe { sorted_branches(tree) };
    assert!(sorted_branches.len() >= 1);

    let init_board = &tree.board;
    let eval_player = init_board.side_to_move();
    let init_value = eval_fun(init_board, eval_player);
    let best_line_prefix = format!("  Best line: [{}] ", init_value);
    use logging::LogLevel;
    logger.writer(LogLevel::Info)
        .map(|writer| print_line_starting(tree, sorted_branches[0], &best_line_prefix, eval_fun, writer));

    debug!(logger, "  Other lines (ordered):");
    logger.writer(LogLevel::Debug)
        .map(|writer|
            for i in 1..sorted_branches.len() {
                print_line_starting(tree, sorted_branches[i], "    ", eval_fun, writer)
            });
}

#[allow(dead_code)]
fn best_line(tree: &LFTree) -> Vec<ChessMove> {
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

fn best_line_full(tree: &LFTree) -> Vec<Either<&LFNode, &LFBranch>> {
    let mut curr_node = tree /*Some(tree)*/;
    let mut line = Vec::new();
    line.push(Left(tree));
    while let Some(branch) = best_branch(curr_node) /*curr_node.and_then(best_branch)*/ {
        //let move_data = best_move_info(curr_node);
        //let move_idx = move_data.mv_idx;
        //let branch = &curr_node.unwrap().moves[move_idx];
        line.push(Right(branch));
        match &branch.child_node {
            Some(child) => {
                line.push(Left(&child));
                curr_node = &child /*branch.child_node.as_ref()*/;
            }
            None => break,
        }
    }
    return line;
}

/* Generation of a dot graph */

pub fn build_dot_graph_from(player: &LFStar, tree: &OpaqueTree) -> dot::Graph {
    build_dot_graph(&tree.0, player.eval)
}

fn build_dot_graph(tree: &LFTree, eval_fun: EvalFun) -> dot::Graph {
    use dot::{NodeProp, EdgeProp, GraphProp};

    let eval_player = tree.board.side_to_move();

    let make_node = |dot_node: dot::Node, search_node: &LFNode| {
        // TODO uncomment this assert
        /*assert!(node_is_consistent(search_node));*/

        let value_now = eval_fun(&search_node.board, eval_player);
        let value_later = best_scores(search_node, eval_fun).get(eval_player);
        let label = format!("now: {}\\nlater: {}", value_now, value_later);
        dot_node.set(
            NodeProp::Label(label))
    };

    let make_edge = |dot_edge: dot::Edge, parent_node: &LFNode, search_edge: &LFBranch| {
        // TODO uncomment this assert
        /*assert!(branch_is_consistent(search_edge, &parent_node.board, eval_fun));*/

        let label = format!("{}\\n{}", search_edge.mv, search_edge.mv_data);
        let curr_player = parent_node.board.side_to_move();
        let edge_score = search_edge.mv_data.get(curr_player);
        let best_score = parent_node.node_data.peek().unwrap().score;

        let mut res_dot_edge = dot_edge.set(EdgeProp::Label(label));
        if edge_score == best_score {
            res_dot_edge = res_dot_edge.set(EdgeProp::KeyValue{
                key:   String::from("penwidth"),
                value: String::from("2")
            });
        }
        else {
            res_dot_edge = res_dot_edge.set(EdgeProp::KeyValue{
                key:   String::from("color"),
                value: String::from("lightgrey")
            })
        }
        return res_dot_edge;
    };

    let make_leaf = |dot_node: dot::Node, parent_node: &LFNode, ending_edge: &LFBranch| {
        let leaf_board = parent_node.board.make_move_new(ending_edge.mv);
        let label = format!("{}", eval_fun(&leaf_board, eval_player));

        dot_node
            .set(NodeProp::Label(label))
            .set(NodeProp::KeyValue {
                key:   String::from("style"),
                value: String::from("dotted")
            })
    };

    searchtree::build_dot_graph(tree, make_node, make_edge, make_leaf)
        .set_graph_global(GraphProp::KeyValue {
            key:   String::from("splines"),
            value: String::from("true")
        })
        .set_node_global(NodeProp::KeyValue {
            key:   String::from("shape"),
            value: String::from("rect")
        })
}
