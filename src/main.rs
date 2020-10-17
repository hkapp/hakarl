mod pgn;
mod eval;
mod play;

fn main() {
    let white = play::evaldriven::classic_eval_player();
    let black = play::random::random_player();
    play::play_game(white, black);
}
