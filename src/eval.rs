use chess::{Board, BoardStatus, Color, Piece, Square};

//type Score = f32;

//const MAX_SCORE: Score = 1000.0;
//const MIN_SCORE: Score = -1000.0;
pub type Score = i16;

const DRAW_SCORE:    Score = 0;
const WINNING_SCORE: Score = Score::MAX;
const LOSING_SCORE:  Score = Score::MIN;
//const MAX_SCORE: Score = 1000.0;
//const MIN_SCORE: Score = -1000.0;

pub type EvalFun = fn (&Board, Color) -> Score;

//trait EvalFun2 {
    //type Score2: std::cmp::PartialOrd;

    //fn eval(board: &Board, color: Color) -> Self::Score2;
//}

//struct EvalFun3 {
    //eval: fn (&Board, Color) -> Score,
    //max_score: Score,
    //min_score: Score,
//}

fn piece_value(piece: Piece) -> Score {
    match piece {
        Piece::Pawn   => 1,
        Piece::Knight => 3,
        Piece::Bishop => 3,
        Piece::Rook   => 5,
        Piece::Queen  => 10,
        Piece::King   => 0,  /* ignored during evaluation */
    }
}

#[allow(unused_parens)]
fn square_value(sq: Square, board: &Board, player: Color) -> Score {
    match board.piece_on(sq) {
        Some(piece) => {
            let color = board.color_on(sq).unwrap();
            let color_mult = if (player == color) { 1 } else { -1 };

            color_mult * piece_value(piece)
        }

        None => 0
    }
}

fn square_based_score(board: &Board, player: Color) -> Score {
    let mut score: Score = 0;

    for sq_ref in chess::ALL_SQUARES.iter() {
        score += square_value(sq_ref.clone(), board, player);
    }

    return score;
}

pub fn classic_eval(board: &Board, player: Color) -> Score {
    match board.status() {
        BoardStatus::Stalemate => DRAW_SCORE,

        BoardStatus::Checkmate =>
            if player == board.side_to_move() { LOSING_SCORE } else { WINNING_SCORE },

        BoardStatus::Ongoing   => square_based_score(board, player),
    }
}
