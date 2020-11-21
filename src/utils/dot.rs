use std::io;
use crate::utils::KeyValue;
use std::fmt;

/* Core DS */

struct Graph {
    nodes:  Vec<Node>,
    edges:  Vec<Edge>,

    graph_name: String,
    graph_config: GraphConfig,
    global_node_config: NodeConfig,
    global_edge_config: EdgeConfig
}

type NodeId = String;
struct Node {
    id:     NodeId,
    config: NodeConfig
}

struct Edge {
    source:      NodeId,
    destination: NodeId,
    config:      EdgeConfig
}

/* Configs */

struct Loose<T> {
    strict_config: T,
    loose_config:  LooseConfig
}

type NodeConfig  = Loose<StrictNodeConfig>;
type EdgeConfig  = Loose<StrictEdgeConfig>;
type GraphConfig = Loose<StrictGraphConfig>;

struct StrictNodeConfig { }
struct StrictEdgeConfig { }
struct StrictGraphConfig { }

type LooseConfig = Vec<KeyValue<String, String>>;

/* Graph API */

impl Graph {

    pub fn new(graph_name: String) -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),

            graph_name,
            graph_config:       Default::default(),
            global_node_config: Default::default(),
            global_edge_config: Default::default(),
        }
    }

    pub fn add_node(&mut self, node: Node) {
        self.nodes.push(node)
    }

    pub fn add_edge(&mut self, edge: Edge) {
        self.edges.push(edge)
    }

    pub fn write_to<W: io::Write>(&self, mut writer: W) -> WriteResult {
        let w = &mut writer;

        writeln!(w, "digraph {} {{", self.graph_name)?;

        write_graph_config(&self.graph_config, w)?;
        write_global_node_config(&self.global_node_config, w)?;
        write_global_edge_config(&self.global_edge_config, w)?;

        write_all_nodes(&self.nodes, w)?;
        write_all_edges(&self.edges, w)?;

        writeln!(w, "}}")
    }
}

impl Default for Graph {
    fn default() -> Self {
        Self::new(String::from("G"))
    }
}

/* Node/Edge API */

impl Node {

    pub fn new(id: NodeId) -> Self {
        Node {
            id,
            config: Default::default()
        }
    }

    pub fn set_conf_str(&mut self, key: String, value: String) {
        self.config.loose_config.push(
            KeyValue { key, value }
        )
    }
}

impl Edge {

    pub fn new(source: NodeId, destination: NodeId) -> Self {
        Edge {
            source,
            destination,
            config: Default::default()
        }
    }

    pub fn set_conf_str(&mut self, key: String, value: String) {
        self.config.loose_config.push(
            KeyValue { key, value }
        )
    }
}

/* File output */

type WriteResult = io::Result<()>;

fn write_all<W, I>(iter: I, writer: &mut W) -> WriteResult
    where
        W: io::Write,
        I: Iterator<Item = String>
{
    for s in iter {
        writeln!(writer, "{}", s)?;
    }
    Ok(())
}

fn write_graph_config<W: io::Write>(conf: &GraphConfig, writer: &mut W) -> WriteResult {
    write_all(
        conf.as_string_pairs()
            .into_iter()
            .map(|KeyValue { key, value }| format_assignment(key, value)),
        writer
    )
}

fn write_global_node_config<W: io::Write>(node_conf: &NodeConfig, writer: &mut W) -> WriteResult {
    writeln!(writer, "node {}", node_conf)
}

fn write_global_edge_config<W: io::Write>(edge_conf: &EdgeConfig, writer: &mut W) -> WriteResult {
    writeln!(writer, "edge {}", edge_conf)
}

fn write_all_nodes<W: io::Write>(nodes: &[Node], writer: &mut W) -> WriteResult {
    write_all(
        nodes.iter()
            .map(|node| format!("{}", node)),
        writer)
}

fn write_all_edges<W: io::Write>(edges: &[Edge], writer: &mut W) -> WriteResult {
    write_all(
        edges.iter()
            .map(|edge| format!("{}", edge)),
        writer)
}

/* Basic formatting */

fn format_assignment(lhs: String, rhs: String) -> String {
    format!("{}={}", lhs, rhs)
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.config.is_empty() {
            write!(f, "{}", self.id)
        }
        else {
            write!(f, "{} {}", self.id, self.config)
        }
    }
}

impl fmt::Display for Edge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.config.is_empty() {
            write!(f, "{} -> {}", self.source, self.destination)
        }
        else {
            write!(f, "{} -> {} {}", self.source, self.destination, self.config)
        }
    }
}

/* Can't derive fmt::Display from Config
 * see https://stackoverflow.com/questions/31082179/is-there-a-way-to-implement-a-trait-on-top-of-another-trait
 */
fn fmt_config<C: Config>(conf: &C, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "[")?;
    let mut sep = "";
    for KeyValue { key, value } in conf.as_string_pairs() {
        write!(f, "{}{}", sep, format_assignment(key, value))?;
        sep = ", ";
    }
    write!(f, "]")
}

impl fmt::Display for NodeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_config(self, f)
    }
}

impl fmt::Display for EdgeConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt_config(self, f)
    }
}

/* Config trait */

trait Config {
    fn as_string_pairs(&self) -> Vec<KeyValue<String, String>>;

    fn is_empty(&self) -> bool;
}

impl<T: Config> Config for Loose<T> {
    fn as_string_pairs(&self) -> Vec<KeyValue<String, String>> {
        let mut res = self.strict_config.as_string_pairs();
        res.extend_from_slice(&self.loose_config);
        return res;
    }

    fn is_empty(&self) -> bool {
        self.loose_config.is_empty() && self.strict_config.is_empty()
    }
}

impl Config for StrictNodeConfig {
    fn as_string_pairs(&self) -> Vec<KeyValue<String, String>> {
        Vec::new()
    }

    fn is_empty(&self) -> bool {
        true
    }
}

impl Config for StrictEdgeConfig {
    fn as_string_pairs(&self) -> Vec<KeyValue<String, String>> {
        Vec::new()
    }

    fn is_empty(&self) -> bool {
        true
    }
}

impl Config for StrictGraphConfig {
    fn as_string_pairs(&self) -> Vec<KeyValue<String, String>> {
        Vec::new()
    }

    fn is_empty(&self) -> bool {
        true
    }
}

/* Default implementations */

impl<T: Default> Default for Loose<T> {
    fn default() -> Self {
        Self {
            loose_config:  Vec::new(),
            strict_config: Default::default()
        }
    }
}

impl Default for StrictNodeConfig {
    fn default() -> Self {
        Self {}
    }
}

impl Default for StrictEdgeConfig {
    fn default() -> Self {
        Self {}
    }
}

impl Default for StrictGraphConfig {
    fn default() -> Self {
        Self {}
    }
}
