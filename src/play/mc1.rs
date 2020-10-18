use chess::{Board, MoveGen, ChessMove, Color};
use super::evaldriven;
use super::evaldriven::EvalPlayer;
use super::ChessPlayer;
use rand::Rng;
use rand::rngs::ThreadRng;
use rand::distributions::{Distribution, WeightedIndex};
use crate::play;

/*********** Structs definition *************/

struct Root {
    init_board: Board,
    root_node:  Node,
}

struct Node {
    moves: Vec<(ChessMove, Stats)>
}

type RunCount = u16;
struct Stats {
    /* Wins and losses are with respect to the player who's got to play
     * in the root, not the current node.
     */
    wins:       RunCount,
    losses:     RunCount,
    stalemates: RunCount,
    tot_games:  RunCount
}

fn new_root(board: &Board) -> Root {
    Root {
        init_board: board.clone(),
        root_node:  new_node(board),
    }
}

fn new_node(board: &Board) -> Node {
    let movegen = MoveGen::new_legal(&board);
    let mv_and_stats = movegen.map(|mv| (mv, Stats::default())).collect();
    Node {
        moves: mv_and_stats,
    }
}

impl std::default::Default for Stats {
    fn default() -> Stats {
        Stats {
            wins:       0,
            losses:     0,
            stalemates: 10, /* TODO make this a constant */
            tot_games:  10
        }
    }
}

/*********** Algorithm implementation *************/

fn chance_to_pick_at_random(stats: &Stats) -> f32 {
    let win_value   = 1.0;
    let stale_value = 0.5;
    let lose_value  = 0.0;

    let tot_value = (win_value * (stats.wins as f32))
                    + (stale_value * (stats.stalemates as f32))
                    + (lose_value * (stats.losses as f32));

    return tot_value / (stats.tot_games as f32);
}

fn pick_node_move<R: Rng>(node: &Node, rng: &mut R) -> usize {
    let weights: Vec<f32> = (&node.moves).into_iter()
                                         .map(|(_, stats)| chance_to_pick_at_random(&stats))
                                         .collect();
    let weighted_dist = WeightedIndex::new(&weights).unwrap();

    weighted_dist.sample(rng)
}

fn update_stats(stats: &mut Stats, player: Color, game_result: Option<Color>) {
    /* The order of field updates will become important once we get to a parallel implementation */
    stats.tot_games += 1;
    match game_result {
        Some(winner) if winner == player     => stats.wins += 1,
        Some(_winner) /*if winner != player*/ => stats.losses += 1,
        None                                 => stats.stalemates += 1,
    };
}

fn run_once<P: ChessPlayer, R: Rng>(
    root:          &mut Root,
    black_rollout: &mut P,
    white_rollout: &mut P,
    rng:           &mut R)
{
    let root_node = &mut root.root_node;
    let move_idx = pick_node_move(&root_node, rng);
    let first_move = root_node.moves[move_idx].0;
    let stats_to_update = &mut root_node.moves[move_idx].1;

    let board_after_move = root.init_board.make_move_new(first_move);
    let game = play::play_game_from(white_rollout, black_rollout, board_after_move);

    update_stats(stats_to_update, root.init_board.side_to_move(), game.winner());
}

fn run_monte_carlo_search<P: ChessPlayer, R: Rng>(
    board:          &Board,
    n_runs:         RunCount,
    white_rollout:  &mut P,
    black_rollout:  &mut P,
    rng:            &mut R)
    -> Root
{
    let mut root = new_root(board);
    for _ in 0..n_runs {
        run_once(&mut root, white_rollout, black_rollout, rng);
    }

    return root;
}

/* See https://www.reddit.com/r/rust/comments/29kia3/no_ord_for_f32/ */
fn max_by_partial_ord<I, F, B>(iter: I, mut f: F) -> Option<I::Item>
    where
        I: Iterator,
        F: FnMut(&I::Item) -> B,
        B: PartialOrd
{
    iter.max_by(|a, b| f(a).partial_cmp(&f(b)).unwrap_or(std::cmp::Ordering::Equal))
}

fn pick_best_move(root: &Root) -> ChessMove {
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
    n_runs:        RunCount,
    rng:           R
}

impl<P: ChessPlayer, R: Rng> ChessPlayer for MonteCarlo1<P, R> {
    fn pick_move(&mut self, board: &Board) -> ChessMove {
        let res_root = run_monte_carlo_search(board,
                                              self.n_runs,
                                              &mut self.white_rollout,
                                              &mut self.black_rollout,
                                              &mut self.rng);
        pick_best_move(&res_root)
    }
}

/*********** Constructors *************/

pub fn monte_carlo1<P: ChessPlayer + Clone>(rollout_player: P, n_runs: RunCount)
    -> MonteCarlo1<P, ThreadRng>
{
    MonteCarlo1::<P, _> {
        white_rollout: rollout_player.clone(),
        black_rollout: rollout_player,
        n_runs,
        rng: rand::thread_rng()
    }
}

pub fn basic_monte_carlo1() -> MonteCarlo1<EvalPlayer, ThreadRng> {
    let n_runs = 100;
    monte_carlo1(evaldriven::classic_eval_player(), n_runs)
}
