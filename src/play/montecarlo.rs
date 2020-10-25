use chess::{Board, MoveGen, ChessMove};
use super::evaldriven;
use super::evaldriven::EvalPlayer;
use super::ChessPlayer;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, WeightedIndex};
use crate::play;
use play::MoveCount;
use std::time::{Duration, Instant};
use std::fmt::Display;

mod stats;
use stats::MoveEval;

/*********** Structs definition *************/

struct Root<S> {
    init_board: Board,
    root_node:  Node<S>,
}

struct Node<S> {
    moves: Vec<(ChessMove, S)>
}

type RunCount = u16;

fn new_root<M: MoveEval>(board: &Board, move_eval: &M) -> Root<M::Stats> {
    Root {
        init_board: board.clone(),
        root_node:  new_node(board, move_eval),
    }
}

fn new_node<M: MoveEval>(board: &Board, move_eval: M) -> Node<M::Stats> {
    let movegen = MoveGen::new_legal(&board);
    let mv_and_stats = movegen.map(|mv| (mv, move_eval.new_stats())).collect();
    Node {
        moves: mv_and_stats,
    }
}

/*********** Algorithm implementation *************/

/*********** Make f32 Ord *************/

#[derive(PartialEq, PartialOrd)]
struct UnsafeOrdF32(f32);

impl Eq for UnsafeOrdF32 {}

impl Ord for UnsafeOrdF32 {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.partial_cmp(&other.0).unwrap()
        //self.partial_cmp(other).unwrap_or(std::cmp::Ordering::Equal)
    }
}

fn pick_node_move<M: MoveEval, R: Rng>(
    node:       &Node<M::Stats>,
    move_eval:  &M,
    rng:        &mut R,
    show_distr: bool)
    -> usize
{
    let weights: Vec<_> = (&node.moves).into_iter()
                                         .map(|(_, stats)| move_eval.eval(&stats))
                                         .collect();
    let weighted_dist = WeightedIndex::new(&weights).unwrap();
    if show_distr {
        println!("Distribution: {:?}", weighted_dist);
    }

    weighted_dist.sample(rng)
}

fn run_once<P: ChessPlayer, M: MoveEval, R: Rng>(
    root:          &mut Root<M::Stats>,
    move_eval:     &mut M,
    black_rollout: &mut P,
    white_rollout: &mut P,
    rollout_depth: MoveCount,
    rng:           &mut R,
    show_distr:    bool)
{
    let root_node = &mut root.root_node;
    let move_idx = pick_node_move(&root_node, move_eval, rng, show_distr);
    let first_move = root_node.moves[move_idx].0;
    let stats_to_update = &mut root_node.moves[move_idx].1;

    let board_after_move = root.init_board.make_move_new(first_move);
    let game = play::play_n_moves(board_after_move,
                                  white_rollout,
                                  black_rollout,
                                  rollout_depth);

    move_eval.update_stats(stats_to_update,
                           root.init_board.side_to_move(),
                           game);
}

fn print_run_info<M, S>(root:      &Root<S>,
                        move_eval: M,
                        run_dur:   Duration,
                        n_runs:    RunCount)
    where
        M: MoveEval<Stats = S>,
        S: Display
{
    let ms_elapsed = run_dur.as_millis();
    println!("Executed {} runs in {}ms", n_runs, ms_elapsed);
    println!("  Average: {:.1} ms per run", (ms_elapsed as f32 / n_runs as f32));
    println!("           {:.1} runs per second", (n_runs as f32 / ms_elapsed as f32) * 1000.);

    //let mut fmt_stats = String::new();
    //let moves = root.root_node.moves.clone();
    let moves = root.root_node.moves.iter();
    let moves_with_values = moves.map(|(m, s)| (m, s, move_eval.eval(&s)));
    //let sorted_moves = moves.map(|(m, s)| (m, s, chance_to_pick_at_random(&s)))
    let mut sorted_moves: Vec<_> = moves_with_values.collect();
    sorted_moves.sort_by_key(|(.., v)| std::cmp::Reverse(UnsafeOrdF32(*v)));
    //sorted_moves.sort_by(|(_ma, _sa, va), (_mb, _sb, vb)| unsafe_cmp_partial_ord(va, vb).reverse());

    //let mut left_padding = String::from("  ");
    let print_count = 3;

    println!("  Best moves:");
    for (mv, stats, mv_value) in sorted_moves.iter().take(print_count) {
        println!("    [{}] {} ~> {}", mv, stats, mv_value);
    }

    println!("  Worst moves:");
    for (mv, stats, mv_value) in sorted_moves.iter()
                                             .rev()
                                             .take(print_count)
                                             .rev()
    {
        println!("    [{}] {} ~> {}", mv, stats, mv_value);
    }
}

fn run_monte_carlo_search<P, M, S, R>(
    board:          &Board,
    move_eval:      &mut M,
    time_budget:    Duration,
    white_rollout:  &mut P,
    black_rollout:  &mut P,
    rollout_depth:  MoveCount,
    rng:            &mut R)
    -> Root<S>
    where
        P: ChessPlayer,
        M: MoveEval<Stats = S>,
        S: Display,
        R: Rng
{
    let start_time = Instant::now();
    let show_distr = false;/*rng.gen_range(0, 20) == 0;*/
    let mut n_runs = 0;

    let mut root = new_root(board, move_eval);
    while start_time.elapsed() < time_budget {
        run_once(&mut root,
                 move_eval,
                 white_rollout,
                 black_rollout,
                 rollout_depth,
                 rng,
                 show_distr);
        n_runs += 1;
    }

    print_run_info(&root, move_eval, start_time.elapsed(), n_runs);

    return root;
}

fn unsafe_cmp_partial_ord<T: PartialOrd>(a: &T, b: &T) -> std::cmp::Ordering {
    a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
}

/* See https://www.reddit.com/r/rust/comments/29kia3/no_ord_for_f32/ */
fn max_by_partial_ord<I, F, B>(iter: I, mut f: F) -> Option<I::Item>
    where
        I: Iterator,
        F: FnMut(&I::Item) -> B,
        B: PartialOrd
{
    iter.max_by(|a, b| unsafe_cmp_partial_ord(&f(a), &f(b)))
    //iter.max_by(|a, b| f(a).partial_cmp(&f(b)).unwrap_or(std::cmp::Ordering::Equal))
}

fn pick_best_move<M: MoveEval>(root: &Root<M::Stats>, move_eval: M) -> ChessMove {
    /* f32 does not implement Ord, only PartialOrd */
    max_by_partial_ord(
        (&root.root_node.moves).into_iter(),
        |(_mv, stats)| move_eval.eval(&stats)
    ).unwrap().0
}

/*********** ChessPlayer definition *************/

pub struct MonteCarlo1<P: ChessPlayer, M: MoveEval, R: Rng> {
    white_rollout: P,
    black_rollout: P,
    rollout_depth: MoveCount,
    move_eval:     M,
    time_budget:   Duration,
    rng:           R,
}

impl<P, M, S, R> ChessPlayer for MonteCarlo1<P, M, R>
    where
        P: ChessPlayer,
        M: MoveEval<Stats = S>,
        S: Display,
        R: Rng
{
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        let res_root =
            run_monte_carlo_search(
                board,
                &mut self.move_eval,
                self.time_budget,
                &mut self.white_rollout,
                &mut self.black_rollout,
                self.rollout_depth,
                &mut self.rng);

        pick_best_move(&res_root, &self.move_eval)
    }
}

/*********** Constructors *************/

pub fn monte_carlo1<P: ChessPlayer + Clone>(
    rollout_player: P,
    time_budget:    Duration,
    rollout_depth:  MoveCount)
    -> MonteCarlo1<P, stats::DefaultEval, ThreadRng>
{
    MonteCarlo1::<P, _, _> {
        white_rollout: rollout_player.clone(),
        black_rollout: rollout_player,
        move_eval:     stats::DefaultEval::default(),
        rollout_depth,
        time_budget,
        rng: rand::thread_rng(),
    }
}

const DEFAULT_TIME_BUDGET: Duration = Duration::from_millis(500);
const DEFAULT_ROLLOUT_DEPTH: MoveCount = 2*20;
pub fn basic_monte_carlo1() -> MonteCarlo1<EvalPlayer, stats::DefaultEval, ThreadRng> {
    monte_carlo1(evaldriven::classic_eval_player(),
                 DEFAULT_TIME_BUDGET,
                 DEFAULT_ROLLOUT_DEPTH)
}
