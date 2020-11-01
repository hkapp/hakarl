use chess::{Board, ChessMove, MoveGen};

/*********** Structs definition *************/

pub struct Node<N, M> {
    pub board:      Board,
    pub node_data:  N,
    pub moves:      Vec<Branch<N, M>>
}

pub struct Branch<N, M> {
    pub mv:         ChessMove,
    pub mv_data:    M,
    pub child_node: Option<Node<N, M>>
}

/*********** API *************/

impl<N, M> Node<N, M> {
    pub fn new<F>(board: Board, node_data: N, mut move_data: F) -> Node<N, M>
        where
            F: FnMut(&Board, ChessMove) -> M
    {
        let branches = MoveGen::new_legal(&board)
                                .map(|mv| Branch::new(mv, move_data(&board, mv)))
                                .collect();
        Node::<N, M> {
            board,
            node_data,
            moves: branches
        }
    }
}

impl<N, M> Branch<N, M> {
    pub fn new(mv: ChessMove, mv_data: M) -> Branch<N, M> {
        Branch::<N, M> {
            mv,
            mv_data,
            child_node: None
        }
    }
}
