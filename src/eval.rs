use chess::{Board, Color, Piece};

//type Score = f32;

//const MAX_SCORE: Score = 1000.0;
//const MIN_SCORE: Score = -1000.0;
type Score = i16;

//const MAX_SCORE: Score = 1000.0;
//const MIN_SCORE: Score = -1000.0;

type EvalFun = fn (&Board, Color) -> Score;

trait EvalFun2 {
    type Score2: std::cmp::PartialOrd;

    fn eval(board: &Board, color: Color) -> Self::Score2;
}

struct EvalFun3 {
    eval: fn (&Board, Color) -> Score,
    max_score: Score,
    min_score: Score,
}

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
pub fn classic_eval(board: &Board, player: Color) -> Score {
    let mut score: Score = 0;
    for sq_ref in chess::ALL_SQUARES.iter() {
        let sq = sq_ref.clone();
        match board.piece_on(sq) {
            Some(piece) => {
                let color = board.color_on(sq).unwrap();
                let color_mult = if (player == color) { 1 } else { -1 };

                score += (color_mult * piece_value(piece));
            },

            None => {},
        }
    }
    //for piece_kind in all_pieces() {
        //let bit_set = board.pieces(piece_kind) // need the color here !
    //}
    return score;
}
