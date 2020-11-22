use chess::{Board, BoardStatus, ChessMove, Color};
use crate::utils;
use crate::logging;

pub mod random;
pub mod evaldriven;
pub mod montecarlo;
pub mod exhaustive;
pub mod astar;
mod searchtree;

pub type Logger = dyn logging::Logger;

pub trait ChessPlayer {
    fn pick_move(&mut self, board: &Board, logger: &mut Logger) -> ChessMove;
}

pub struct Game {
    pub init_board:  Board,
    pub final_board: Board,
    pub moves:       Vec<ChessMove>,
}

impl Game {
    pub fn new() -> Self {
        Game {
            init_board:  Board::default(),
            final_board: Board::default(),
            moves:       Vec::new()
        }
    }

    #[allow(dead_code)]
    pub fn is_over(&self) -> bool {
        self.final_board.status() != BoardStatus::Ongoing
    }

    pub fn winner(&self) -> Option<Color> {
        match self.final_board.status() {
            BoardStatus::Checkmate => Some(!self.final_board.side_to_move()),
            _                      => None
        }
    }

    pub fn result_for(&self, player: Color) -> Option<GameResult> {
        match self.final_board.status() {
            BoardStatus::Checkmate =>
                if self.final_board.side_to_move() != player { Some(GameResult::Win) }
                else { Some(GameResult::Lose) },

            BoardStatus::Stalemate => Some(GameResult::Draw),

            BoardStatus::Ongoing => None
        }
    }

    pub fn play_move(&mut self, mv: ChessMove) {
        self.final_board = self.final_board.make_move_new(mv);
        self.moves.push(mv);
    }

    pub fn continue_playing<P1: ChessPlayer, P2: ChessPlayer>(
        &mut self,
        white:     &mut P1,
        black:     &mut P2,
        max_moves: MoveCount,
        logger:    &mut Logger)
    {
        let mut game_played = play_n_moves(self.final_board, white, black, max_moves, logger);

        self.final_board = game_played.final_board;
        self.moves.append(&mut game_played.moves);
    }
}

pub enum GameResult {
    Win,
    Draw,
    Lose,
}

pub fn play_game<P1: ChessPlayer, P2: ChessPlayer>(
    white:  &mut P1,
    black:  &mut P2,
    logger: &mut Logger)
    -> Game
{
    play_game_from(Board::default(), white, black, logger)
}

const DEFAULT_MAX_MOVES: MoveCount = 200; /* 100 turns */
pub fn play_game_from<P1: ChessPlayer, P2: ChessPlayer>(
    start_pos: Board,
    white:     &mut P1,
    black:     &mut P2,
    logger:    &mut Logger)
    -> Game
{
    play_n_moves(start_pos, white, black, DEFAULT_MAX_MOVES, logger)
}

type MoveCount = u8;

pub fn play_n_moves<P1: ChessPlayer, P2: ChessPlayer>(
    start_pos: Board,
    white:     &mut P1,
    black:     &mut P2,
    max_moves: MoveCount,
    logger:    &mut Logger)
    -> Game
{
    let mut board = start_pos.clone();
    let mut move_list = Vec::new();
    let max_moves = max_moves as usize;  /* convenience cast */

    while board.status() == BoardStatus::Ongoing && move_list.len() < max_moves {
        let mv = match board.side_to_move() {
            Color::White => white.pick_move(&board, logger),
            Color::Black => black.pick_move(&board, logger)
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

pub fn replay_game(game: Game) -> impl Iterator<Item = (Board, ChessMove)> {
    utils::iter::stateful_map(
        game.init_board,
        game.moves.into_iter(),
        |board, mv| board.make_move_new(*mv)
    )
}
