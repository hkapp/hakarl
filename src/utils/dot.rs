use std::io;
use crate::utils::display;
use std::fmt;

/* Core DS */

pub struct Graph {
    nodes:  Vec<Node>,
    edges:  Vec<Edge>,

    graph_name: String,
    graph_config: GraphConfig,
    global_node_config: NodeConfig,
    global_edge_config: EdgeConfig
}

pub type NodeId = String;
pub struct Node {
    id:     NodeId,
    config: NodeConfig
}

pub struct Edge {
    source:      NodeId,
    destination: NodeId,
    config:      EdgeConfig
}

/* Configs (old) */

/*struct Loose<T> {
    strict_config: T,
    loose_config:  LooseConfig
}

type NodeConfig  = Loose<StrictNodeConfig>;
type EdgeConfig  = Loose<StrictEdgeConfig>;
type GraphConfig = Loose<StrictGraphConfig>;

struct StrictNodeConfig { }
struct StrictEdgeConfig { }
struct StrictGraphConfig { }

type LooseConfig = Vec<KeyValue<String, String>>;*/

/* Configs */

struct Config<T> {
    props: Vec<T>
}

type NodeConfig = Config<NodeProp>;
type EdgeConfig = Config<EdgeProp>;
type GraphConfig = Config<GraphProp>;

#[allow(dead_code)]
pub enum NodeProp {
    Label(String),
    KeyValue { key: String, value: String }
}

#[allow(dead_code)]
pub enum EdgeProp {
    Label(String),
    KeyValue { key: String, value: String }
}

#[allow(dead_code)]
pub enum GraphProp {
    KeyValue { key: String, value: String }
}

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

    #[allow(dead_code)]
    pub fn set_graph_global(mut self, prop: GraphProp) -> Self {
        self.graph_config.set(prop);
        return self;
    }

    pub fn set_node_global(mut self, prop: NodeProp) -> Self {
        self.global_node_config.set(prop);
        return self;
    }

    #[allow(dead_code)]
    pub fn set_edge_global(mut self, prop: EdgeProp) -> Self {
        self.global_edge_config.set(prop);
        return self;
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

    /*pub fn set_conf_str(&mut self, key: String, value: String) {
        self.config.loose_config.push(
            KeyValue { key, value }
        )
    }*/

    pub fn set(mut self, prop: NodeProp) -> Self {
        self.config.set(prop);
        return self;
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

    /*pub fn set_conf_str(&mut self, key: String, value: String) {
        self.config.loose_config.push(
            KeyValue { key, value }
        )
    }*/

    pub fn set(mut self, prop: EdgeProp) -> Self {
        self.config.set(prop);
        return self;
    }
}

/* Config API */

impl<T> Config<T> {
    fn is_empty(&self) -> bool {
        self.props.is_empty()
    }

    fn set(&mut self, prop: T) {
        self.props.push(prop)
    }
}

/* File output */

type WriteResult = io::Result<()>;

fn write_all<W, I, D>(iter: I, writer: &mut W) -> WriteResult
    where
        W: io::Write,
        I: Iterator<Item = D>,
        D: fmt::Display
{
    for item in iter {
        writeln!(writer, "{}", item)?;
    }
    Ok(())
}

fn write_graph_config<W: io::Write>(conf: &GraphConfig, writer: &mut W) -> WriteResult {
    write_all(conf.props.iter(), writer)
}

fn write_global_node_config<W: io::Write>(node_conf: &NodeConfig, writer: &mut W) -> WriteResult {
    writeln!(writer, "node {}", node_conf)
}

fn write_global_edge_config<W: io::Write>(edge_conf: &EdgeConfig, writer: &mut W) -> WriteResult {
    writeln!(writer, "edge {}", edge_conf)
}

fn write_all_nodes<W: io::Write>(nodes: &[Node], writer: &mut W) -> WriteResult {
    write_all(nodes.iter(), writer)
}

fn write_all_edges<W: io::Write>(edges: &[Edge], writer: &mut W) -> WriteResult {
    write_all(edges.iter(), writer)
}

/* Basic formatting */

fn format_assignment(lhs: &str, rhs: &str) -> String {
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

impl<T: fmt::Display> fmt::Display for Config<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let formatted_props =
            self.props
                .iter()
                .map(|p| format!("{}", p));

        write!(f, "{}", display::enclosed_join(formatted_props, "[", ", ", "]"))
    }
}

impl fmt::Display for NodeProp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_assignment(self.key_str(), &self.val_str()))
    }
}

impl fmt::Display for EdgeProp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_assignment(self.key_str(), &self.val_str()))
    }
}

impl fmt::Display for GraphProp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", format_assignment(self.key_str(), &self.val_str()))
    }
}

fn format_label(label: &str) -> String {
    /* FIXME need to escape too */
    format!("\"{}\"", label)
}

/* Property trait */

trait Property {
    fn key_str(&self) -> &str;
    fn val_str(&self) -> String;
}

impl Property for NodeProp {
    fn key_str(&self) -> &str {
        match self {
            NodeProp::Label(_)             => "label",
            NodeProp::KeyValue { key, .. } => key
        }
    }

    fn val_str(&self) -> String {
        match self {
            NodeProp::Label(lab)             => format_label(lab),
            NodeProp::KeyValue { value, .. } => String::from(value)
        }
    }
}

impl Property for EdgeProp {
    fn key_str(&self) -> &str {
        match self {
            EdgeProp::Label(_)             => "label",
            EdgeProp::KeyValue { key, .. } => key
        }
    }

    fn val_str(&self) -> String {
        match self {
            EdgeProp::Label(lab)             => format_label(lab),
            EdgeProp::KeyValue { value, .. } => String::from(value)
        }
    }
}

impl Property for GraphProp {
    fn key_str(&self) -> &str {
        match self {
            GraphProp::KeyValue { key, .. } => key
        }
    }

    fn val_str(&self) -> String {
        match self {
            GraphProp::KeyValue { value, .. } => String::from(value)
        }
    }
}

/* Can't derive fmt::Display from Config
 * see https://stackoverflow.com/questions/31082179/is-there-a-way-to-implement-a-trait-on-top-of-another-trait
 */
/*fn fmt_config<C: Config>(conf: &C, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
}*/

/* Config trait */

/*trait Config {
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
}*/

/* Default implementations */

impl<T> Default for Config<T> {
    fn default() -> Self {
        Self {
            props: Vec::new()
        }
    }
}
