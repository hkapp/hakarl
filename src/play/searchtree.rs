use chess::{Board, ChessMove};
use crate::utils::dot;
use crate::eval::EvalFun;

/*********** Structs definition *************/

pub type Tree<N, M> = Node<N, M>;

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

/********** TreePlayer **********/

pub trait TreePlayer {
    type NodeData;
    type BranchData;

    fn build_tree(&mut self, board: &Board, logger: &mut super::Logger) -> Tree<Self::NodeData, Self::BranchData>;

    fn best_move(&self, tree: &Tree<Self::NodeData, Self::BranchData>) -> ChessMove;

}

impl<T: TreePlayer> super::ChessPlayer for T {
    fn pick_move(&mut self, board: &Board, logger: &mut super::Logger) -> ChessMove {
        let t = self.build_tree(board, logger);
        self.best_move(&t)
    }
}

/********** Dot generation **********/

pub fn build_dot_graph<N, M, FN, FE, FL>(tree: &Tree<N, M>, make_node: FN, make_edge: FE, make_leaf: Option<FL>)
    -> dot::Graph
    where
        FN: Fn(dot::Node, &Node<N, M>) -> dot::Node,
        FE: Fn(dot::Edge, &Node<N, M>, &Branch<N, M>) -> dot::Edge,
        FL: Fn(dot::Node, &Node<N, M>, &Branch<N, M>) -> dot::Node,
{
    fn node_id_from_board(board: &Board) -> dot::NodeId {
        /* FIXME need unique ids here */
        format!("{}", board.get_hash())
    }

    fn build_dot_node<F>(board: &Board, make_node: F) -> dot::Node
        where F: Fn(dot::Node) -> dot::Node,
    {
        let node_id = node_id_from_board(board);

        make_node(dot::Node::new(node_id))
    }

    fn rec_add_node<N, M, FN, FE, FL>(
        node:      &Node<N, M>,
        make_node: &FN,
        make_edge: &FE,
        make_leaf: &Option<FL>,
        graph:     &mut dot::Graph)
        where
            FN: Fn(dot::Node, &Node<N, M>) -> dot::Node,
            FE: Fn(dot::Edge, &Node<N, M>, &Branch<N, M>) -> dot::Edge,
            FL: Fn(dot::Node, &Node<N, M>, &Branch<N, M>) -> dot::Node,
    {
        let dot_node = build_dot_node(&node.board, |n| make_node(n, node));
        graph.add_node(dot_node);

        for branch in node.moves.iter() {
            rec_add_edge(node, branch, make_node, make_edge, make_leaf, graph);
        }
    };

    fn rec_add_edge<N, M, FN, FE, FL>(
        parent_node: &Node<N, M>,
        edge:        &Branch<N, M>,
        make_node:   &FN,
        make_edge:   &FE,
        make_leaf:   &Option<FL>,
        graph:       &mut dot::Graph)
        where
            FN: Fn(dot::Node, &Node<N, M>) -> dot::Node,
            FE: Fn(dot::Edge, &Node<N, M>, &Branch<N, M>) -> dot::Edge,
            FL: Fn(dot::Node, &Node<N, M>, &Branch<N, M>) -> dot::Node,
    {
        let prev_board = &parent_node.board;
        let next_board = match edge.child_node.as_ref() {
            Some(node) => node.board.clone(),
            None       => prev_board.make_move_new(edge.mv),
        };
        let next_board = &next_board;

        let src_id = node_id_from_board(prev_board);
        let dst_id = node_id_from_board(next_board);

        let dot_edge = make_edge(dot::Edge::new(src_id, dst_id),
                                 parent_node, edge);

        graph.add_edge(dot_edge);

        match edge.child_node.as_ref() {
            Some(child_node) => rec_add_node(child_node, make_node, make_edge, make_leaf, graph),
            None             => if let Some(make_child) = make_leaf {
                let dest_node = build_dot_node(prev_board, |n| make_child(n, parent_node, edge));
                graph.add_node(dest_node)
            }
        }
    };

    let mut graph = dot::Graph::default();

    rec_add_node(tree, &make_node, &make_edge, &make_leaf, &mut graph);

    return graph;
}

#[allow(dead_code)]
pub fn basic_dot_graph<N, M>(tree: &Tree<N, M>, eval_fun: EvalFun) -> dot::Graph {

    let eval_player = tree.board.side_to_move();

    let make_node = |dot_node: dot::Node, search_node: &Node<N, M>| {
        use dot::NodeProp;

        let label = format!("{}", eval_fun(&search_node.board, eval_player));
        dot_node.set(
            NodeProp::Label(label))
    };

    let make_edge = |dot_edge: dot::Edge, _parent_node: &Node<N, M>, search_edge: &Branch<N, M>| {
        use dot::EdgeProp;

        let label = format!("{}", search_edge.mv);
        dot_edge.set(
            EdgeProp::Label(label))
    };

    let make_leaf = |dot_node: dot::Node, parent_node: &Node<N, M>, ending_edge: &Branch<N, M>| {
        use dot::NodeProp;

        let leaf_board = parent_node.board.make_move_new(ending_edge.mv);
        let label = format!("{}", eval_fun(&leaf_board, eval_player));

        dot_node
            .set(NodeProp::Label(label))
            .set(NodeProp::KeyValue {
                key:   String::from("style"),
                value: String::from("dotted")
            })
    };

    build_dot_graph(tree, make_node, make_edge, Some(make_leaf))
}
