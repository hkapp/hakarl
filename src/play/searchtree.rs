use chess::{Board, ChessMove};

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
    /*pub fn new<F>(board: Board, node_data: N, mut move_data: F) -> Node<N, M>
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
    }*/

    pub fn count_nodes(&self) -> u32 {
        //let mut count = 1;
        //for branch in self.moves {
            //count += branch.child_node.map(|child| child.count_nodes())
                                      //.unwrap_or(0);
        //}
        //return count;

        let rec_count: u32 = self.children().map(|child| child.count_nodes())
                                            .sum();
        return rec_count + 1;
    }

    pub fn depth(&self) -> u16 {
        let rec_depth = self.children().map(|child| child.depth())
                                       .max()
                                       .unwrap_or(0);
        return rec_depth + 1;
    }

    pub fn children(&self) -> impl Iterator<Item=&'_ Node<N,M>> {
        self.moves.iter()
                  .filter_map(|branch| branch.child_node.as_ref())
    }
}

impl<N, M> Branch<N, M> {
    /*pub fn new(mv: ChessMove, mv_data: M) -> Branch<N, M> {
        Branch::<N, M> {
            mv,
            mv_data,
            child_node: None
        }
    }*/
}
