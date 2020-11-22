use chess::{self, ChessMove, Piece, Board, Color, Square, File};
use crate::play;
use std::{str, convert};

/********** WRITE **********/

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

fn try_castle_format(fmt_mv: FmtMove) -> Option<String> {
    let (mv, moved_piece) = fmt_mv;

    if moved_piece == Piece::King && mv.get_source().get_file() == File::E {
        /* Checking the rank is pointless, as the files check correspond
         * to a 2-hop or 3-hop move, which can only be done via castling
         */
        match mv.get_dest().get_file() {
            File::G => Some(String::from("O-O")),
            File::C => Some(String::from("O-O-O")),
            _       => None
        }

    }
    else {
        None
    }
}

fn gen_move(fmt_mv: FmtMove) -> String {
    /* Check for castle move, which has special format */
    let castle_rep = try_castle_format(fmt_mv);
    if castle_rep.is_some() {
        return castle_rep.unwrap();
    }

    let (mv, moved_piece) = fmt_mv;
    /* TODO make this code use 'format' to append to the string directly? */
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

/********** READ **********/

/* Parser infrastructure */

type ParseErr = String;
type ParseRes<T> = Result<T, ParseErr>;

struct ParsedPrefix<'a, T> {
    value:   T,
    rem_str: &'a str
}

impl<'a, T> ParsedPrefix<'a, T> {
    fn as_full_parse(self) -> ParseRes<T> {
        if self.rem_str.is_empty() {
            Ok(self.value)
        }
        else {
            Err(format!("Not at end of string, the following text remains: \"{}\"", self.rem_str))
        }
    }
}

trait Parse: Sized {
    fn parse<'a>(s: &'a str) -> ParseRes<ParsedPrefix<'a, Self>>;
}

struct Parser<'a> {
    rem_str: &'a str
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Parser {
            rem_str: s
        }
    }

    fn parse<T: Parse>(&mut self) -> ParseRes<T> {
        T::parse(self.rem_str)
            .map(|ParsedPrefix { value, rem_str: rem_after_parse }| {
                self.rem_str = rem_after_parse;
                value
            })
        /*match T::parse(self.rem_str) {
            Ok(ParsedPrefix { value, rem_str: rem_after_parse }) => {
                self.rem_str = rem_after_parse;
                Ok(value)
            }
            Err(err) => Err(err)
        }*/
    }

    /*fn finalize<T>(self, value: T) -> ParseRes<T> {
    }*/

    fn finalize_prefix<T>(self, value: T) -> ParseRes<ParsedPrefix<'a, T>> {
        Ok(ParsedPrefix {
            value,
            rem_str: self.rem_str
        })
    }
}

/* helper for elements that can be parsed as a single character */
trait ParseChar: Sized {
    fn parse_char(c: char) -> ParseRes<Self>;
}

impl<T: ParseChar> Parse for T {
    fn parse<'a>(s: &'a str) -> ParseRes<ParsedPrefix<'a, T>> {
        match s.chars().next() {
            None             => Err(String::from("Input string is empty")),
            Some(first_char) =>
                T::parse_char(first_char)
                    .map(|value| ParsedPrefix {
                        value,
                        rem_str: &s[1..]
                    })
        }
    }
}

/* Define FromStr for anything that implements Parse */
/* Can't do because of E[0210] ? */
/*impl<T: Parse> str::FromStr for T {
    type Err = ParseErr;

    fn from_str(s: &str) -> ParseRes<T> {
        T::parse(s)
            .and_then(|ParsedPrefix { value, rem_str }|
                if rem_str.is_empty() {
                    Ok(T)
                }
                else {
                    Err(format!("Not at end of string, the following text remains: \"{}\"", rem_str))
                }
            )
    }
}*/

/* Data structures */
/* TODO lift up and share with 'Write' part */

struct PGNPiece(chess::Piece);
struct PGNSquare(chess::Square);
struct PGNFile(chess::File);
struct PGNRank(chess::Rank);

/* Parsing Piece */

fn parse_piece(c: char) -> Option<chess::Piece> {
    use chess::Piece::*;
    match c {
        'P' => Some(Pawn),
        'N' => Some(Knight),
        'B' => Some(Bishop),
        'R' => Some(Rook),
        'Q' => Some(Queen),
        'K' => Some(King),
        _   => None
    }

}

/*impl str::FromStr for PGNPiece {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 1 {
            return Err(format!("\"{}\" is too long to be a piece", s));
        }
        else {
            let first_char = s.chars().next().unwrap();
            parse_piece(first_char)
                .ok_or(format!("\'{}\' does not represent any chess piece", first_char))
        }
    }
}*/
impl ParseChar for PGNPiece {
    fn parse_char(c: char) -> ParseRes<Self> {
        match parse_piece(c) {
            Some(p) => Ok(PGNPiece(p)),
            None    => Err(format!("\'{}\' does not represent any chess piece", c))
        }
    }
}

/* Parsing Square */

fn parse_file(c: char) -> Option<chess::File> {
    use chess::File;
    match c {
        'a' => Some(File::A),
        'b' => Some(File::B),
        'c' => Some(File::C),
        'd' => Some(File::D),
        'e' => Some(File::E),
        'f' => Some(File::F),
        'g' => Some(File::G),
        'h' => Some(File::H),
        _   => None
    }
}

impl ParseChar for PGNFile {
    fn parse_char(c: char) -> ParseRes<Self> {
        match parse_file(c) {
            Some(f) => Ok(PGNFile(f)),
            None    => Err(format!("\'{}\' does not represent any chess file", c))
        }
    }
}

fn parse_rank(c: char) -> Option<chess::Rank> {
    use chess::Rank;
    match c {
        '1' => Some(Rank::First),
        '2' => Some(Rank::Second),
        '3' => Some(Rank::Third),
        '4' => Some(Rank::Fourth),
        '5' => Some(Rank::Fifth),
        '6' => Some(Rank::Sixth),
        '7' => Some(Rank::Seventh),
        '8' => Some(Rank::Eighth),
        _   => None
    }
}

impl ParseChar for PGNRank {
    fn parse_char(c: char) -> ParseRes<Self> {
        match parse_rank(c) {
            Some(r) => Ok(PGNRank(r)),
            None    => Err(format!("\'{}\' does not represent any chess rank", c))
        }
    }
}

impl Parse for PGNSquare {
    fn parse<'a>(s: &'a str) -> ParseRes<ParsedPrefix<'a, Self>> {
        let mut parser = Parser::new(s);
        let file = parser.parse::<PGNFile>()?;
        let rank = parser.parse::<PGNRank>()?;
        let square = Square::make_square(rank.into(), file.into());
        parser.finalize_prefix(PGNSquare(square))
    }
}

/* Complete move parsing */

struct PGNMove {
    piece:     chess::Piece,
    base_move: chess::ChessMove,
}

impl str::FromStr for PGNMove {
    type Err = ParseErr;

    fn from_str(s: &str) -> ParseRes<Self> {
        parse_castle_move(s)
            .or_else(|_| parse_regular_move_or_promote(s))
    }
}

fn parse_regular_move_prefix(s: &str) -> ParseRes<ParsedPrefix<PGNMove>> {
    let mut parser  = Parser::new(s);
    let piece       = parser.parse::<PGNPiece>()?;
    let source      = parser.parse::<PGNSquare>()?;
    let dest        = parser.parse::<PGNSquare>()?;

    let mv = PGNMove {
        piece:     piece.into(),
        base_move: chess::ChessMove::new(
            source.into(),
            dest.into(),
            None
        )
    };

    parser.finalize_prefix(mv)
}

fn parse_regular_move(s: &str) -> ParseRes<PGNMove> {
    parse_regular_move_prefix(s)?
        .as_full_parse()
}

fn parse_regular_move_or_promote(s: &str) -> ParseRes<PGNMove> {
    let prefix = parse_regular_move_prefix(s)?;
    let rem_str = prefix.rem_str;
    let mut regular_move = prefix.value;

    if rem_str.is_empty() {
        Ok(regular_move)
    }
    else {
        // this should be a promotion
        if rem_str.chars().next().unwrap() != '=' {
            return Err(format!("Invalid promote format: '{}'", s));
        }

        let promote_piece = PGNPiece::parse(&rem_str[1..])?.as_full_parse()?;
        let base_move = regular_move.base_move;
        let promote_move = chess::ChessMove::new(
            base_move.get_source(),
            base_move.get_dest(),
            Some(promote_piece.into())
        );

        regular_move.base_move = promote_move;
        Ok(regular_move)
    }
}

fn parse_castle_move(s: &str) -> ParseRes<PGNMove> {
    Err(String::from("Not yet implemented"))
}

/* Complete game parsing */

pub fn read_pgn(input: &str) -> ParseRes<play::Game> {
    let mut game = play::Game::new();
    let mut turn = 1;

    let mut chunks = input.split_whitespace();
    while let Some(turn_indicator) = chunks.next() {
        /* Validate the turn_indicator */
        let expected_turnind = format!("{}.", turn);
        assert_eq!(turn_indicator, &expected_turnind);
        turn += 1;

        /* Extract white's move (mandatory) */
        let white_str = chunks.next().ok_or("White's move is mandatory each turn")?;
        let white_move = white_str.parse::<PGNMove>()?;
        game.play_move(white_move.into());

        /* Extract black's move (optional) */
        let black_str = match chunks.next() {
            Some(s) => s,
            None    => return Ok(game),  /* EOS */
        };
        let black_move = black_str.parse::<PGNMove>()?;
        game.play_move(black_move.into());
    }

    Ok(game)
}

/* Conversions */

impl convert::From<PGNRank> for chess::Rank {
    fn from(pgn_rank: PGNRank) -> chess::Rank {
        pgn_rank.0
    }
}

impl convert::From<PGNFile> for chess::File {
    fn from(pgn_file: PGNFile) -> chess::File {
        pgn_file.0
    }
}

impl convert::From<PGNSquare> for chess::Square {
    fn from(pgn_sq: PGNSquare) -> chess::Square {
        pgn_sq.0
    }
}

impl convert::From<PGNPiece> for chess::Piece {
    fn from(pgn_piece: PGNPiece) -> chess::Piece {
        pgn_piece.0
    }
}

impl convert::From<PGNMove> for chess::ChessMove {
    fn from(pgn_mv: PGNMove) -> chess::ChessMove {
        pgn_mv.base_move
    }
}
