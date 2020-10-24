use chess::{Board, BoardStatus, ChessMove, Color};

pub mod random;
pub mod evaldriven;
pub mod montecarlo;

pub trait ChessPlayer {
    fn pick_move(&mut self, board: &Board) -> ChessMove;
}

pub struct Game {
    pub init_board:  Board,
    pub final_board: Board,
    pub moves:       Vec<ChessMove>,
}

impl Game {
    pub fn is_over(&self) -> bool {
        self.final_board.status() != BoardStatus::Ongoing
    }

    pub fn winner(&self) -> Option<Color> {
        match self.final_board.status() {
            BoardStatus::Checkmate => Some(!self.final_board.side_to_move()),
            _                      => None
        }
    }
}

pub fn play_game<P1: ChessPlayer, P2: ChessPlayer>(white: &mut P1, black: &mut P2) -> Game {
    play_game_from(white, black, Board::default())
}

pub fn play_game_from<P1: ChessPlayer, P2: ChessPlayer>(
    white:     &mut P1,
    black:     &mut P2,
    start_pos: Board)
    -> Game
{
    let mut board = start_pos.clone();
    let mut move_list = Vec::new();
    let max_moves = 200;

    while board.status() == BoardStatus::Ongoing && move_list.len() < max_moves {
        let mv = match board.side_to_move() {
            Color::White => white.pick_move(&board),
            Color::Black => black.pick_move(&board)
        };
        move_list.push(mv);
        board = board.make_move_new(mv);
    }

    return Game {
        init_board: start_pos,
        final_board: board,
        moves: move_list
    };
}

#[allow(dead_code)]
pub fn play_random_game() {
    let mut white = random::random_player();
    let mut black = random::random_player();
    play_game(&mut white, &mut black);
}
