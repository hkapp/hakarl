use chess::{Color, BoardStatus};
use crate::play;
use crate::play::{Game, GameResult};
use crate::eval;
use eval::EvalFun;
use std::fmt;

use super::RunCount;

/*********** Stats trait *************/

pub type Value = f32;

pub type DefaultEval = EvalTrace;

pub trait MoveEval {

    type Stats;

    fn new_stats(&self) -> Self::Stats;

    fn update_stats(&mut self,
                    stats:  &mut Self::Stats,
                    player: Color,
                    game:   play::Game);

    fn eval(&self, stats: &Self::Stats) -> Value;

}

#[allow(unused_variables)]
impl<'a, M> MoveEval for &'a M
    where M: MoveEval
{

    type Stats = M::Stats;

    fn new_stats(&self) -> Self::Stats {
        (*self).new_stats()
    }

    fn update_stats(&mut self,
                    stats:  &mut Self::Stats,
                    player: Color,
                    game:   play::Game)
    {
        panic!("Can't implement 'MoveEval::update_stats' for '&S'");
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        (*self).eval(stats)
    }

}

#[allow(unused_variables)]
impl<'a, M> MoveEval for &'a mut M
    where M: MoveEval
{

    type Stats = M::Stats;

    fn new_stats(&self) -> Self::Stats {
        M::new_stats(self)
    }

    fn update_stats(&mut self,
                    stats:  &mut Self::Stats,
                    player: Color,
                    game:   play::Game)
    {
        M::update_stats(self, stats, player, game);
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        M::eval(self, stats)
    }

}

/*********** BasicStats *************/

#[derive(Clone)]
pub struct BasicStats {
    /* Wins and losses are with respect to the player who's got to play
     * in the root, not the current node.
     */
    wins:       RunCount,
    losses:     RunCount,
    stalemates: RunCount,
    tot_games:  RunCount
}

impl BasicStats {

    fn update(&mut self, player: Color, game_result: play::Game) {
        /* The order of field updates will become important once we get to a parallel implementation */
        self.tot_games += 1;
        match game_result.winner() {
            Some(winner) if winner == player      => self.wins += 1,
            Some(_winner) /*if winner != player*/ => self.losses += 1,
            None                                  => self.stalemates += 1,
        };
    }

}

impl fmt::Display for BasicStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}W/{}L/{}D", self.wins, self.losses, self.stalemates)
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

/*********** WinDrawAverage *************/

pub struct WinDrawAverage;

impl MoveEval for WinDrawAverage {

    type Stats = BasicStats;

    fn new_stats(&self) -> Self::Stats {
        BasicStats {
            wins:       0,
            losses:     0,
            stalemates: 10,
            tot_games:  10
        }
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        let win_value   = 5.0;
        let stale_value = 1.0;
        let lose_value  = 0.0;

        let wins       = stats.wins as f32;
        let losses     = stats.losses as f32;
        let stalemates = stats.stalemates as f32;
        let tot_games  = stats.tot_games as f32;

        let tot_value = (win_value * wins)
                        + (stale_value * stalemates)
                        + (lose_value * losses);

        return tot_value / tot_games;
    }

    fn update_stats(&mut self,
                    stats:       &mut Self::Stats,
                    player:      Color,
                    game_result: play::Game)
    {
        stats.update(player, game_result);
    }

}

/*********** Powers *************/

pub struct Powers;

impl MoveEval for Powers {

    type Stats = BasicStats;

    fn new_stats(&self) -> Self::Stats {
        BasicStats {
            wins:       0,
            losses:     0,
            stalemates: 0,
            tot_games:  0
        }
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        let w = stats.wins as f32;
        let l = stats.losses as f32;
        let d = stats.stalemates as f32;

        fn sigma(x: f32, y: f32, d: f32) -> f32 {
            fn q(x: f32, d: f32) -> f32 {
                fn k(x: f32, d: f32) -> f32 {
                    (x + 1.) / (x + d + 1.)
                }

                x + k(x, d)
            }

            fn nu(x: f32, y: f32) -> f32 {
                (2. * x) / (x + y)
            }

            q(x, d).powf(nu(x, y))
        }

        let c = 3.;
        if w == 0. && l == 0. {
            c
        }
        else {
            c * (sigma(w, l, d) / sigma(l, w, d))
        }
    }

    fn update_stats(&mut self,
                    stats:       &mut Self::Stats,
                    player:      Color,
                    game_result: play::Game)
    {
        stats.update(player, game_result);
    }

}

/*********** Limits *************/

pub struct Limits;

impl MoveEval for Limits {

    type Stats = BasicStats;

    fn new_stats(&self) -> Self::Stats {
        BasicStats {
            wins:       0,
            losses:     0,
            stalemates: 1,
            tot_games:  1
        }
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        let w   = stats.wins as f32;
        let l   = stats.losses as f32;
        let d   = stats.stalemates as f32;
        let tot = stats.tot_games as f32;

        fn lim(x: f32) -> f32 {
            (x + 1.) / (x + 2.)
        }

        let lim_w = w * lim(w);
        let lim_l = l * (1. - lim(l));
        let lim_d = d / 2.;

        (lim_w + lim_d + lim_l) / tot
    }

    fn update_stats(&mut self,
                    stats:       &mut Self::Stats,
                    player:      Color,
                    game_result: play::Game)
    {
        stats.update(player, game_result);
    }

}

impl std::default::Default for Limits {
    fn default() -> Self {
        Limits
    }
}

/*********** EvalUndecided *************/

pub struct TraceAverage {
    n_wins:    RunCount,
    n_lose:    RunCount,
    n_draws:   RunCount,
    draw_eval: Value,
    n_runs:    RunCount,
}

pub struct EvalUndecided {
    eval_board: eval::EvalFun,
}
/* this will also require to refactor the Stats trait and split it in two
 * Separate the Stats and MoveEval part
 * The MoveEval then stores the configuration of the evaluator
 */

impl MoveEval for EvalUndecided {

    type Stats = TraceAverage;

    fn new_stats(&self) -> Self::Stats {
        TraceAverage {
            n_wins:    0,
            n_lose:    0,
            n_draws:   1,
            draw_eval: 0.,
            n_runs:    1,
        }
    }

    fn update_stats(&mut self, stats: &mut Self::Stats, player: Color, game: play::Game) {
        let game_result = game.result_for(player).unwrap_or(GameResult::Draw);

        match game_result {
            GameResult::Win  => stats.n_wins += 1,

            GameResult::Lose => stats.n_lose += 1,

            GameResult::Draw => {
                let init_val  = (self.eval_board)(&game.init_board, player);
                let final_val = (self.eval_board)(&game.final_board, player);
                let val_diff = (final_val as Value) - (init_val as Value);

                stats.draw_eval += val_diff;
                stats.n_draws += 1;
            }
        };

        stats.n_runs += 1;
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        let win_part  = 100. * (stats.n_wins as Value);  /* TODO get the constant from the evaluation function */
        let lose_part = 0.;                     /* TODO re-evaluate this */
        let draw_part = (50. * (stats.n_draws as Value)) + stats.draw_eval;  /* TODO here too */

        (win_part + lose_part + draw_part) / (stats.n_runs as Value)
    }

}

impl std::default::Default for EvalUndecided {
    fn default() -> Self {
        EvalUndecided {
            eval_board: eval::classic_eval,
        }
    }
}

impl fmt::Display for TraceAverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}W/{}L/{}D({:+.2})", self.n_wins,
                                         self.n_lose,
                                         self.n_draws,
                                         self.draw_eval / (self.n_draws as Value))
    }
}
/*********** EvalTrace *************/

struct TraceStat {
    tot_value: Value,
    n_runs:    RunCount,
}

pub struct TraceStats {
    wins:     TraceStat,
    losses:   TraceStat,
    draws:    TraceStat,
    tot_runs: RunCount,
}

impl TraceStat {
    fn update(&mut self, player: Color, game: Game, eval_fun: EvalFun, discount_factor: f32) {
        let mut game_val = DiscountAvg::new(discount_factor);

        for (board, _mv) in play::replay_game(game) {
            let board_val = match board.status() {
                BoardStatus::Ongoing => 50. + (eval_fun(&board, player) as f32),
                BoardStatus::Stalemate => 50.,
                BoardStatus::Checkmate =>
                    if board.side_to_move() != player { 100. }
                    else { 0. }
            };

            game_val.add(board_val);
        }

        self.tot_value += game_val.avg();
        //let mut game_val = 0.;
        //let mut curr_discount = 1.;
        //let mut discount_sum = 0.;

        //for (board, _mv) in play::replay_game(game) {
            //let board_val = match board.status() {
                //BoardStatus::Ongoing => 50. + (eval_fun(&board, player) as f32),
                //BoardStatus::Stalemate => 50.,
                //BoardStatus::Checkmate =>
                    //if board.side_to_move() != player { 100. }
                    //else { 0. }
            //}
            //game_val += board_val * curr_discount;
            //discount_sum += curr_discount;
            //curr_discount *= discount_factor;
        //}

        //self.tot_value += (game_val / discount_sum);
        self.n_runs += 1;
    }
}

impl std::default::Default for TraceStat {
    fn default() -> Self {
        TraceStat {
            tot_value: 0.,
            n_runs:    0,
        }
    }
}

impl TraceStats {
    fn update(&mut self, player: Color, game: Game, eval_fun: EvalFun, discount_factor: f32) {

        let game_result = game.result_for(player).unwrap_or(GameResult::Draw);

        let corr_trc = match game_result {
            GameResult::Win  => &mut self.wins,
            GameResult::Lose => &mut self.losses,
            GameResult::Draw => &mut self.draws,
        };
        corr_trc.update(player, game, eval_fun, discount_factor);

        self.tot_runs += 1;
    }
}

pub struct EvalTrace {
    eval_board:      eval::EvalFun,
    discount_factor: f32
}
/* this will also require to refactor the Stats trait and split it in two
 * Separate the Stats and MoveEval part
 * The MoveEval then stores the configuration of the evaluator
 */

struct DiscountAvg {
    curr_sum:        f32,
    curr_discount:   f32,
    discount_sum:    f32,
    discount_factor: f32
}

impl DiscountAvg {
    fn new(discount_factor: f32) -> Self {
        DiscountAvg {
            curr_sum:        0.,
            curr_discount:   1.,
            discount_sum:    0.,
            discount_factor: discount_factor
        }
    }

    #[allow(unused_parens)]
    fn add(&mut self, val: f32) {
        self.curr_sum += (self.curr_discount * val);
        self.discount_sum += self.curr_discount;
        self.curr_discount *= self.discount_factor;
    }

    fn avg(&self) -> f32 {
        self.curr_sum / self.discount_sum
    }
}

impl MoveEval for EvalTrace {

    type Stats = TraceStats;

    fn new_stats(&self) -> Self::Stats {
        let default_draw_stats = TraceStat {
            tot_value: 50.,
            n_runs:    1
        };

        TraceStats {
            wins:     TraceStat::default(),
            losses:   TraceStat::default(),
            draws:    default_draw_stats,
            tot_runs: 1
        }
    }

    fn update_stats(&mut self, stats: &mut Self::Stats, player: Color, game: play::Game) {
        stats.update(player, game, self.eval_board, self.discount_factor);
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        fn part_of(trc: &TraceStat) -> Value {
            trc.tot_value //* (trc.n_runs as Value)
        }

        let win_part  = part_of(&stats.wins);
        let lose_part = part_of(&stats.losses);
        let draw_part = part_of(&stats.draws);

        (win_part + lose_part + draw_part) / (stats.tot_runs as Value)
    }

}

const DEFAULT_DISCOUNT_FACTOR: f32 = 0.90;
impl std::default::Default for EvalTrace {
    fn default() -> Self {
        EvalTrace {
            eval_board:      eval::classic_eval,
            discount_factor: DEFAULT_DISCOUNT_FACTOR,
        }
    }
}

impl fmt::Display for TraceStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fn fmt_trace(trc: &TraceStat, ind: char, is_last: bool, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            let mut res = write!(f, "{}{}", trc.n_runs, ind);
            if trc.n_runs > 0 {
                res = write!(f, " ({:.2})", trc.tot_value / (trc.n_runs as Value));
            }
            if !is_last {
                res = write!(f, " ");
            }
            res
        }

        fmt_trace(&self.wins,   'W', false, f)
            .and_then(|_| fmt_trace(&self.losses, 'L', false, f))
            .and_then(|_| fmt_trace(&self.draws,  'D', true,  f))
    }
}
