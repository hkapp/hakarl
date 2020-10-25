use chess::Color;
use crate::play;
use crate::play::GameResult;
use crate::eval;
use std::fmt;

use super::RunCount;

/*********** Stats trait *************/

pub type Value = f32;


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

pub type DefaultEval = Limits;

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
    tot_value: Value,
    n_runs:    super::RunCount,
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
            tot_value: 50.,  /* TODO replace with a constant */
            n_runs:    1,
        }
    }

    fn eval(&self, stats: &Self::Stats) -> Value {
        stats.tot_value / (stats.n_runs as Value)
    }

    fn update_stats(&mut self, stats: &mut Self::Stats, player: Color, game: play::Game) {
        let game_result = game.result_for(player).unwrap_or(GameResult::Draw);

        stats.tot_value += match game_result {
            GameResult::Win  => 100., /* TODO get the value from the evaluation function */

            GameResult::Lose => 0.,  /* is this good? */

            GameResult::Draw => {
                let init_val  = (self.eval_board)(&game.init_board, player);
                let final_val = (self.eval_board)(&game.final_board, player);

                (final_val as Value) - (init_val as Value)
            }
        };

        stats.n_runs += 1;
    }

}

//impl fmt::Display for TraceAverage {
    //fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "{}", self.0)
    //}
//}
