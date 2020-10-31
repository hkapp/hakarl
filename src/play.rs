use chess::{Board, BoardStatus, ChessMove, Color};

pub mod random;
pub mod evaldriven;
pub mod montecarlo;
pub mod exhaustive;

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

    pub fn result_for(&self, player: Color) -> Option<GameResult> {
        match self.final_board.status() {
            BoardStatus::Checkmate =>
                if self.final_board.side_to_move() != player { Some(GameResult::Win) }
                else { Some(GameResult::Lose) },

            BoardStatus::Stalemate => Some(GameResult::Draw),

            BoardStatus::Ongoing => None
        }
    }

    pub fn continue_playing<P1: ChessPlayer, P2: ChessPlayer>(
        &mut self,
        white:     &mut P1,
        black:     &mut P2,
        max_moves: MoveCount)
    {
        let mut game_played = play_n_moves(self.final_board, white, black, max_moves);

        self.final_board = game_played.final_board;
        self.moves.append(&mut game_played.moves);
    }
}

pub enum GameResult {
    Win,
    Draw,
    Lose,
}

pub fn play_game<P1: ChessPlayer, P2: ChessPlayer>(white: &mut P1, black: &mut P2) -> Game {
    play_game_from(Board::default(), white, black)
}

const DEFAULT_MAX_MOVES: MoveCount = 200; /* 100 turns */
pub fn play_game_from<P1: ChessPlayer, P2: ChessPlayer>(
    start_pos: Board,
    white:     &mut P1,
    black:     &mut P2)
    -> Game
{
    play_n_moves(start_pos, white, black, DEFAULT_MAX_MOVES)
}

type MoveCount = u8;

pub fn play_n_moves<P1: ChessPlayer, P2: ChessPlayer>(
    start_pos: Board,
    white:     &mut P1,
    black:     &mut P2,
    max_moves: MoveCount)
    -> Game
{
    let mut board = start_pos.clone();
    let mut move_list = Vec::new();
    let max_moves = max_moves as usize;  /* convenience cast */

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

pub struct FoldMap<S, I, F> {
    state:     S,
    iter:      I,
    upd_state: F
}

pub fn stateful_map<S, I, F>(init_state: S, iter: I, upd_state: F) -> FoldMap<S, I, F>
    where
        I: Iterator,
        F: FnMut(&S, &I::Item) -> S,
        S: Clone
{
    FoldMap {
        state: init_state,
        iter,
        upd_state
    }
}

impl<S, I, F> Iterator for FoldMap<S, I, F>
    where
        I: Iterator,
        F: FnMut(&S, &I::Item) -> S,
        S: Clone
{
    type Item = (S, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(a) => {
                self.state = (self.upd_state)(&self.state, &a);
                Some((self.state.clone(), a))
            }

            None => None
        }
    }
}

pub fn replay_game(game: Game) -> impl Iterator<Item = (Board, ChessMove)> {
    stateful_map(
        game.init_board,
        game.moves.into_iter(),
        |board, mv| board.make_move_new(*mv)
    )
}
