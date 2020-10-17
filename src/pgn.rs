use chess::{ChessMove, Piece, Board, Color, Square};

fn moved_piece(board: &Board, mv: ChessMove) -> Piece {
    board.piece_on(mv.get_source()).unwrap()
}

fn piece_fmt(piece: Piece) -> String {
    piece.to_string(Color::White)
}

fn pos_fmt(sq: Square) -> String {
    format!("{}", sq)
}

fn promote_fmt(mv: ChessMove) -> Option<String> {
    match mv.get_promotion() {
        Some(new_piece) => {
            let prefix = '=';
            let mut piece_rep = piece_fmt(new_piece);
            piece_rep.insert(0, prefix);
            return Some(piece_rep);
        }

        None => None
    }
}

fn gen_move(fmt_mv: FmtMove) -> String {
    let (mv, moved_piece) = fmt_mv;

    let source_rep = pos_fmt(mv.get_source());
    let dest_rep   = pos_fmt(mv.get_dest());

    let piece_rep = piece_fmt(moved_piece);

    let promote_rep = promote_fmt(mv).unwrap_or_default();

    format!("{}{}{}{}", piece_rep, source_rep, dest_rep, promote_rep)
}

type FmtMove = (ChessMove, Piece);

fn gen_turn(white: FmtMove, black: Option<FmtMove>, turn: u8) -> String {
    let white_rep = gen_move(white);

    let black_rep = match black {
        Some(black_fmt) => gen_move(black_fmt),
        None            => String::new()
    };

    return format!("{}. {} {} ", turn, white_rep, black_rep)
}

pub struct PGNBuilder {
    turn: u8,
    white_move: Option<FmtMove>,
    buffer: String
}

impl PGNBuilder {
    fn new() -> PGNBuilder {
        PGNBuilder {
            turn: 1,
            white_move: None,
            buffer: String::new()
        }
    }

    fn push_move(&mut self, board: &Board, mv: ChessMove) {
        match self.white_move {
            Some(white_move) => {
                /* We now have both black and white moves, generate the turn */
                let black_mv = mv;
                let black_piece = moved_piece(board, black_mv);
                let black_move = Some((black_mv, black_piece));

                let turn_rep = gen_turn(white_move, black_move, self.turn);
                self.buffer.push_str(&turn_rep);
                self.turn += 1;
                self.white_move = None;
            }

            None => {
                /* This is a white move. Cache it and wait for black move */
                self.white_move = Some((mv, moved_piece(board, mv)));
            }
        }
    }

    fn to_string(&self) -> String {
        match self.white_move {
            Some(white_move) => {
                /* Buffer is not up-to-date. Generate the last incomplete move and return */
                let mut res = self.buffer.clone();
                let black_move = None;
                let turn_rep = gen_turn(white_move, black_move, self.turn);
                res.push_str(&turn_rep);
                return res;
            }

            None => {
                /* Buffer is up-to-date. Just copy it */
                return self.buffer.clone();
            }
        }
    }
}

pub fn basic_pgn(move_list: &[ChessMove]) -> String {
    let mut board = Board::default();
    let mut pgn_fmt = PGNBuilder::new();

    for mv_ref in move_list {
        let mv = mv_ref.clone();
        pgn_fmt.push_move(&board, mv);
        board = board.make_move_new(mv);
    }

    return pgn_fmt.to_string();
}

//pub fn basic_pgn(move_list: &[ChessMove]) -> String {
    //let mut mv_idx: usize = 0;
    //let mut res = String::new();

    //while mv_idx < move_list.len() {
        //let turn: u8 = ((mv_idx / 2) + 1) as u8;
        //let white_move = &move_list[mv_idx];
        //let black_move = move_list.get(mv_idx+1);
        //let turn_rep = gen_turn(white_move, black_move, turn);
        //res.push_str(&turn_rep);
        //mv_idx += 2;
    //}

    //return res;
//}
