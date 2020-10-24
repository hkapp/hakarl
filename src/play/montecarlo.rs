use chess::{Board, MoveGen, ChessMove};
use super::evaldriven;
use super::evaldriven::EvalPlayer;
use super::ChessPlayer;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, WeightedIndex};
use crate::play;
use std::time::{Duration, Instant};
use std::fmt::Display;

mod stats;
use stats::Stats;

/*********** Structs definition *************/

struct Root<S: Stats> {
    init_board: Board,
    root_node:  Node<S>,
}

struct Node<S: Stats> {
    moves: Vec<(ChessMove, S)>
}

type RunCount = u16;

//#[derive(Clone)]
//struct Stats {
    // /* Wins and losses are with respect to the player who's got to play
     //* in the root, not the current node.
     //*/
    //wins:       RunCount,
    //losses:     RunCount,
    //stalemates: RunCount,
    //tot_games:  RunCount
//}

fn new_root<S: Stats>(board: &Board) -> Root<S> {
    Root {
        init_board: board.clone(),
        root_node:  new_node(board),
    }
}

fn new_node<S: Stats>(board: &Board) -> Node<S> {
    let movegen = MoveGen::new_legal(&board);
    let mv_and_stats = movegen.map(|mv| (mv, S::new())).collect();
    Node {
        moves: mv_and_stats,
    }
}

// This value depends on the assumptions made by the current function
// in chance_to_pick_at_random.
// When changing the function, also change this value.
//const STATS_INIT_DRAWS: RunCount = 1;
//impl std::default::Default for Stats {
    //fn default() -> Stats {
        //Stats {
            //wins:       0,
            //losses:     0,
            //stalemates: STATS_INIT_DRAWS,
            //tot_games:  STATS_INIT_DRAWS
        //}
    //}
//}

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

fn chance_to_pick_at_random<S: Stats>(s: &S) -> stats::Value {
    s.value()
}

fn pick_node_move<S: Stats, R: Rng>(node: &Node<S>, rng: &mut R, show_distr: bool) -> usize
{
    let weights: Vec<_> = (&node.moves).into_iter()
                                         .map(|(_, stats)| chance_to_pick_at_random(&stats))
                                         .collect();
    let weighted_dist = WeightedIndex::new(&weights).unwrap();
    if show_distr {
        println!("Distribution: {:?}", weighted_dist);
    }

    weighted_dist.sample(rng)
}

fn run_once<P: ChessPlayer, S: Stats, R: Rng>(
    root:          &mut Root<S>,
    black_rollout: &mut P,
    white_rollout: &mut P,
    rng:           &mut R,
    show_distr:    bool)
{
    let root_node = &mut root.root_node;
    let move_idx = pick_node_move(&root_node, rng, show_distr);
    let first_move = root_node.moves[move_idx].0;
    let stats_to_update = &mut root_node.moves[move_idx].1;

    let board_after_move = root.init_board.make_move_new(first_move);
    let game = play::play_game_from(white_rollout, black_rollout, board_after_move);

    stats_to_update.update(root.init_board.side_to_move(), game);
}

fn print_run_info<S: Stats + Display>(root: &Root<S>, run_dur: Duration, n_runs: RunCount) {
    let ms_elapsed = run_dur.as_millis();
    println!("Executed {} runs in {}ms", n_runs, ms_elapsed);
    println!("  Average: {:.1} ms per run", (ms_elapsed as f32 / n_runs as f32));
    println!("           {:.1} runs per second", (n_runs as f32 / ms_elapsed as f32) * 1000.);

    //let mut fmt_stats = String::new();
    //let moves = root.root_node.moves.clone();
    let moves = root.root_node.moves.iter();
    let moves_with_values = moves.map(|(m, s)| (m, s, chance_to_pick_at_random(&s)));
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

fn run_monte_carlo_search<P: ChessPlayer, S: Stats + Display, R: Rng>(
    board:          &Board,
    time_budget:    Duration,
    white_rollout:  &mut P,
    black_rollout:  &mut P,
    rng:            &mut R)
    -> Root<S>
{
    let start_time = Instant::now();
    let show_distr = false;/*rng.gen_range(0, 20) == 0;*/
    let mut n_runs = 0;

    let mut root = new_root(board);
    while start_time.elapsed() < time_budget {
        run_once(&mut root, white_rollout, black_rollout, rng, show_distr);
        n_runs += 1;
    }

    print_run_info(&root, start_time.elapsed(), n_runs);

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

fn pick_best_move<S: Stats>(root: &Root<S>) -> ChessMove {
    /* f32 does not implement Ord, only PartialOrd */
    max_by_partial_ord(
        (&root.root_node.moves).into_iter(),
        |(_mv, stats)| chance_to_pick_at_random(&stats)
    ).unwrap().0
}

/*********** ChessPlayer definition *************/

pub struct MonteCarlo1<P: ChessPlayer, R: Rng> {
    white_rollout: P,
    black_rollout: P,
    time_budget:   Duration,
    rng:           R
}

impl<P: ChessPlayer, R: Rng> ChessPlayer for MonteCarlo1<P, R> {
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        let res_root = run_monte_carlo_search::<_, stats::DefaultStats, _>(board,
                                              self.time_budget,
                                              &mut self.white_rollout,
                                              &mut self.black_rollout,
                                              &mut self.rng);
        pick_best_move(&res_root)
    }
}

/*********** Constructors *************/

pub fn monte_carlo1<P: ChessPlayer + Clone>(rollout_player: P, time_budget: Duration)
    -> MonteCarlo1<P, ThreadRng>
{
    MonteCarlo1::<P, _> {
        white_rollout: rollout_player.clone(),
        black_rollout: rollout_player,
        time_budget,
        rng: rand::thread_rng()
    }
}

pub fn basic_monte_carlo1() -> MonteCarlo1<EvalPlayer, ThreadRng> {
    let time_budget = Duration::from_millis(500);
    monte_carlo1(evaldriven::classic_eval_player(), time_budget)
}
