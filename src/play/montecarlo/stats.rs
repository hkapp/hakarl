use chess::Color;
use crate::play;
use std::fmt;

use super::RunCount;

/*********** Stats trait *************/

pub type Value = f32;

pub trait Stats {

    fn new() -> Self;

    fn value(&self) -> Value;

    fn update(&mut self, player: Color, game_result: play::Game);

}

#[allow(unused_variables)]
impl<'a, S> Stats for &'a S
    where S: Stats
{

    fn new() -> Self {
        panic!("Can't implement 'Stats::new()' for &S")
    }

    fn value(&self) -> Value {
        (*self).value()
    }

    fn update(&mut self, player: Color, game_result: play::Game) {
        panic!("Can't implement 'Stats::update()' for &S");
    }

}

pub type DefaultStats = Limits;

/*********** BasicStats *************/

#[derive(Clone)]
struct BasicStats {
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

pub struct WinDrawAverage(BasicStats);

impl Stats for WinDrawAverage {

    fn new() -> Self {
        WinDrawAverage (
            BasicStats {
                wins:       0,
                losses:     0,
                stalemates: 10,
                tot_games:  10
            }
        )
    }

    fn value(&self) -> Value {
        let win_value   = 5.0;
        let stale_value = 1.0;
        let lose_value  = 0.0;

        let WinDrawAverage(stats) = self;
        let wins       = stats.wins as f32;
        let losses     = stats.losses as f32;
        let stalemates = stats.stalemates as f32;
        let tot_games  = stats.tot_games as f32;

        let tot_value = (win_value * wins)
                        + (stale_value * stalemates)
                        + (lose_value * losses);

        return tot_value / tot_games;
    }

    fn update(&mut self, player: Color, game_result: play::Game) {
        self.0.update(player, game_result);
    }

}

impl fmt::Display for WinDrawAverage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/*********** Powers *************/

pub struct Powers(BasicStats);

impl Stats for Powers {

    fn new() -> Self {
        Powers (
            BasicStats {
                wins:       0,
                losses:     0,
                stalemates: 0,
                tot_games:  0
            }
        )
    }

    fn value(&self) -> Value {
        let Powers(stats) = self;
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

    fn update(&mut self, player: Color, game_result: play::Game) {
        self.0.update(player, game_result);
    }

}

impl fmt::Display for Powers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/*********** Limits *************/

pub struct Limits(BasicStats);

impl Stats for Limits {

    fn new() -> Self {
        Limits (
            BasicStats {
                wins:       0,
                losses:     0,
                stalemates: 1,
                tot_games:  1
            }
        )
    }

    fn value(&self) -> Value {
        let Limits(stats) = self;
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

    fn update(&mut self, player: Color, game_result: play::Game) {
        self.0.update(player, game_result);
    }

}

impl fmt::Display for Limits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/*********** Algorithm implementation *************/


//#[allow(dead_code)]
//fn chance_to_pick_at_random(stats: &Stats) -> f32 {
    //// A first tentative as a simple average of classical game scores
    //fn f1(wins: f32, stalemates: f32, losses: f32, tot_games: f32) -> f32 {
        //let win_value   = 5.0;
        //let stale_value = 1.0;
        //let lose_value  = 0.0;

        //let tot_value = (win_value * wins)
                        //+ (stale_value * stalemates)
                        //+ (lose_value * losses);

        //return tot_value / tot_games;
    //}

    //// A second attempt based on powers
    //fn f2(w: f32, d: f32, l: f32) -> f32 {
        //fn sigma(x: f32, y: f32, d: f32) -> f32 {
            //fn q(x: f32, d: f32) -> f32 {
                //fn k(x: f32, d: f32) -> f32 {
                    //(x + 1.) / (x + d + 1.)
                //}

                //x + k(x, d)
            //}

            //fn nu(x: f32, y: f32) -> f32 {
                //(2. * x) / (x + y)
            //}

            //q(x, d).powf(nu(x, y))
        //}

        //let c = 3.;
        //match (w, d, l) {
            //(0., _, 0.) => c,
            //_           => c * (sigma(w, l, d) / sigma(l, w, d))
        //}
    //}

    //// A third attempt based on averaging ]0; 1[ values
    //fn f3(w: f32, d: f32, l: f32, tot: f32) -> f32 {
        //fn lim(x: f32) -> f32 {
            //(x + 1.) / (x + 2.)
        //}

        //let lim_w = w * lim(w);
        //let lim_l = l * (1. - lim(l));
        //let lim_d = d / 2.;

        //(lim_w + lim_d + lim_l) / tot
    //}

    //let w = stats.wins as f32;
    //let d = stats.stalemates as f32;
    //let l = stats.losses as f32;
    //let tot = stats.tot_games as f32;

    //// Here we need to add a discount factor with the length the game in case of win / lose
    //f3(w, d, l, tot)
//}

//fn update_stats(stats: &mut Stats, player: Color, game_result: Option<Color>) {
     // /* The order of field updates will become important once we get to a parallel implementation */
    //stats.tot_games += 1;
    //match game_result {
        //Some(winner) if winner == player     => stats.wins += 1,
        //Some(_winner) /*if winner != player*/ => stats.losses += 1,
        //None                                 => stats.stalemates += 1,
    //};
//}



//fn unsafe_cmp_partial_ord<T: PartialOrd>(a: &T, b: &T) -> std::cmp::Ordering {
    //a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
//}

// /* See https://www.reddit.com/r/rust/comments/29kia3/no_ord_for_f32/ */
//fn max_by_partial_ord<I, F, B>(iter: I, mut f: F) -> Option<I::Item>
    //where
        //I: Iterator,
        //F: FnMut(&I::Item) -> B,
        //B: PartialOrd
//{
    //iter.max_by(|a, b| unsafe_cmp_partial_ord(&f(a), &f(b)))
    ////iter.max_by(|a, b| f(a).partial_cmp(&f(b)).unwrap_or(std::cmp::Ordering::Equal))
//}
