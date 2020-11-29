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

/********** Dot generation **********/

pub fn build_dot_graph<N, M, FN, FE, FL>(tree: &Tree<N, M>, make_node: FN, make_edge: FE, make_leaf: FL)
    -> dot::Graph
    where
        FN: Fn(dot::Node, &Node<N, M>) -> dot::Node,
        FE: Fn(dot::Edge, &Node<N, M>, &Branch<N, M>) -> dot::Edge,
        FL: Fn(dot::Node, &Node<N, M>, &Branch<N, M>) -> dot::Node,
{
    fn node_id_for<T>(any_ref: &T) -> dot::NodeId {
        /* Printing the actual pointer should be enough to guarantee uniqueness */
        /* turns out the pointer for the leaves are all the same (which makes sense) */
        let mut hex_str = format!("{:p}", any_ref);
        hex_str.remove(0);  // drop the initial '0', dot doesn't like it
        return hex_str;
    }

    fn new_node<T>(any_ref: &T) -> dot::Node {
        let node_id = node_id_for(any_ref);

        dot::Node::new(node_id)
    }

    //fn build_dot_node<FN>(search_node: &SearchNode, make_node: F) -> dot::Node
        //where F: Fn(dot::Node) -> dot::Node,
    //{
        //let node_id = node_id_for(board);

        //make_node(dot::Node::new(node_id))
    //}

    fn rec_add_node<N, M, FN, FE, FL>(
        node:      &Node<N, M>,
        make_node: &FN,
        make_edge: &FE,
        make_leaf: &FL,
        graph:     &mut dot::Graph)
        where
            FN: Fn(dot::Node, &Node<N, M>) -> dot::Node,
            FE: Fn(dot::Edge, &Node<N, M>, &Branch<N, M>) -> dot::Edge,
            FL: Fn(dot::Node, &Node<N, M>, &Branch<N, M>) -> dot::Node,
    {
        let dot_node = make_node(new_node(node), node);
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
        make_leaf:   &FL,
        graph:       &mut dot::Graph)
        where
            FN: Fn(dot::Node, &Node<N, M>) -> dot::Node,
            FE: Fn(dot::Edge, &Node<N, M>, &Branch<N, M>) -> dot::Edge,
            FL: Fn(dot::Node, &Node<N, M>, &Branch<N, M>) -> dot::Node,
    {
        //let prev_board = &parent_node.board;
        //let next_board = &next_board;

        let src_id = node_id_for(parent_node);
        let dst_id = match edge.child_node.as_ref() {
            Some(child_node) => node_id_for(child_node),
            None             => node_id_for(edge),
        };

        let dot_edge = make_edge(dot::Edge::new(src_id, dst_id),
                                 parent_node, edge);

        graph.add_edge(dot_edge);

        match edge.child_node.as_ref() {
            Some(child_node) => rec_add_node(child_node, make_node, make_edge, make_leaf, graph),
            None             => {
                let dest_node = make_leaf(new_node(edge), parent_node, edge);
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
    use dot::{NodeProp, EdgeProp, GraphProp};

    let eval_player = tree.board.side_to_move();

    let make_node = |dot_node: dot::Node, search_node: &Node<N, M>| {
        let label = format!("{}", eval_fun(&search_node.board, eval_player));
        dot_node.set(
            NodeProp::Label(label))
    };

    let make_edge = |dot_edge: dot::Edge, _parent_node: &Node<N, M>, search_edge: &Branch<N, M>| {
        let label = format!("{}", search_edge.mv);
        dot_edge.set(
            EdgeProp::Label(label))
    };

    let make_leaf = |dot_node: dot::Node, parent_node: &Node<N, M>, ending_edge: &Branch<N, M>| {
        let leaf_board = parent_node.board.make_move_new(ending_edge.mv);
        let label = format!("{}", eval_fun(&leaf_board, eval_player));

        dot_node
            .set(NodeProp::Label(label))
            .set(NodeProp::KeyValue {
                key:   String::from("style"),
                value: String::from("dotted")
            })
    };

    build_dot_graph(tree, make_node, make_edge, make_leaf)
        .set_graph_global(GraphProp::KeyValue {
            key:   String::from("splines"),
            value: String::from("false")
        })
        .set_node_global(NodeProp::KeyValue {
            key:   String::from("shape"),
            value: String::from("circle")
        })
}
