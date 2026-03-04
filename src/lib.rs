//! # Graph Representations
//!
//! In the following we denote the number of vertices by `n` and the number of
//! edges by `m`. Vertices are indexed from `0` to `n - 1` and represented as
//! `usize`.
//!
//! ## Adjacency Sets for Sparse Graphs
//!
//! A graph is commonly called sparse if the number of edges grows at most
//! linearly or sub-polonomially in the number of vertices, i.e. $m = O(n)$.
//!
//! *   `Vec<T : AdjacencySet = FxHashSet<usize>>>`: This is the default
//!     representation of sparse graphs. It is optimized for fast adjacency
//!     checks and updates. For applications that require fast
//!     adjacency checks but no updates on graphs small maximum degrees the
//!     use of `Vec<sparse::adjacency_list::SortedVector>` may be good choice.
//!     When neither fast adjacency checks nor updates are needed the use of
//!     `Vec<Vec<usize>>` may be suitable.
//!
//! *   `Vec<usize>`: For regular graphs the adjancency lists can be stored in a
//!     flat vector. For high degree the list must be sorted for efficient
//!     adjacency checks. This is not needed if edges are only iterated over.
//!     In the sorted case updates are best performed in bulk and lists are
//!     normalized after all updates for a list have been applied.
//!
//! *   `[usize; n]` For static regular graphs with small maximum degree.
//!
//! ## Adjacency Matrices for Dense Graphs
//!
//! TODO

pub mod sparse;
pub mod named;
pub mod random;
pub mod dot;

/// A trait for tagging graph types. This is used for type-level dispatch and
/// enforcing constraints. This will not be needed once Rust support for
/// const generics is stabilized.
pub trait GraphType {}
pub struct Directed;
pub struct Undirected;
impl GraphType for Directed {}
impl GraphType for Undirected {}


#[derive(Debug, Clone)]
pub enum DfsEvent {
    /// An edge from `source` to `target` was discovered.
    Edge(usize, usize),
    /// A vertex was visited for the first time.
    Pre(usize),
    /// A vertex was popped from the stack.
    Post(usize),
}

/// Interface for directed graphs.
///
/// Vertices are represented as `usize` indices from `0` to `order() - 1`.
/// Directed edges are represented as pairs of vertices `(source, target)`.
///
/// # Note to Implementors
///
/// Two graphs are equal when they have the same set of vertices and the same
/// set of edges. The implementation of `PartialEq` and `Eq` must be compatible
/// with this definition.
///
/// The default implementatios are provided as semantic references and are
/// generally not efficient. They should be overridden by implementors for
/// better performance.
///
pub trait Graph: PartialEq + Eq + Clone {
    type Type: GraphType;
    const IS_DIRECTED: bool;
    fn empty(order: usize) -> Self;
    fn from_edges(edges: impl IntoIterator<Item = (usize, usize)>) -> Self;
    fn from_edges_with_order(order: usize, edges: impl IntoIterator<Item = (usize, usize)>) -> Self;
    fn from_adjacency_lists(adjacency_lists: impl IntoIterator<Item = impl IntoIterator<Item = usize>>) -> Self;

    fn vertices(&self) -> impl Iterator<Item = usize>;
    fn edges(&self) -> impl Iterator<Item = (usize, usize)>;

    fn order(&self) -> usize {
        self.vertices().count()
    }

    fn size(&self) -> usize {
        self.edges().count()
    }

    fn is_empty(&self) -> bool {
        self.size() == 0
    }

    fn is_complete(&self) -> bool {
        self.size() == self.complete_edge_count()
    }

    fn adjacents(&self, vertex: usize) -> impl Iterator<Item = usize> {
        self.edges().filter_map(move |(src, trg)| if src == vertex { Some(trg) } else { None })
    }

    fn is_edge(&self, source: usize, target: usize) -> bool {
        self.adjacents(source).any(|v| v == target)
    }

    fn is_reachable(&self, source: usize, target: usize) -> bool;

    /// Depth-first search with edge, pre-order, and post-order hooks.
    ///
    /// # Arguments
    /// *   `vertex`: The starting vertex for the DFS.
    /// *   `hook`: A closure that is called for each DFS event. The closure
    ///     receives a `DfsEvent` enum value and returns a `bool`.
    ///     For `Edge` events, returning `false` prunes the target vertex
    ///     from the search. For `Pre` and `Post` events, the return value
    ///     is currently ignored.
    ///
    fn dfs<F: FnMut(DfsEvent) -> bool>(&self, vertex: usize, hook: F);

    fn complete_edge_count(&self) -> usize;

    fn equal<G>(&self, other: &G) -> bool
    where
        G: Graph,
        G: Graph<Type = Self::Type>
    {
        self.order() == other.order()
        && self.size() == other.size()

        // for implementations it may be more efficient to sort all edges into a
        // vector and compare the vectors.
        && self.edges().all(|e| other.is_edge(e.0, e.1))
    }
}

pub trait MutableGraph: Graph {
    /// Adds an edge from `source` to `target`. Returns `true` if the edge was
    /// added and `false` if the edge already existed.
    ///
    fn add_edge(&mut self, source: usize, target: usize) -> bool;

    /// Removes the edge from `source` to `target`. Returns `true` if the edge
    /// was removed and `false` if the edge did not exist.
    ///
    fn remove_edge(&mut self, source: usize, target: usize) -> bool;
}

pub trait DirectedMutableGraph: MutableGraph {
    /// Transforms the graph into its symmetric hull.
    ///
    fn symmetric_hull_mut(&mut self);
}

/// The class of directed graphs.
///
/// A directed graph is a tuple $(V, E)$ where $V$ is a set of vertices and $E
/// \subseteq V \times V$ is a *set* of directed edges. In particular, a directed
/// graph may contain loops, i.e. edges of the from $(v, v)$ from a some vertex
/// to itself.
///
pub trait DirectedGraph
    where Self: Graph<Type = Directed>
{
    fn out_degree(&self, vertex: usize) -> usize {
        self.adjacents(vertex).count()
    }

    fn in_degree(&self, vertex: usize) -> usize {
        self.edges().filter(|&(_, trg)| trg == vertex).count()
    }

    fn max_out_degree(&self) -> usize {
        self.vertices().map(|v| self.out_degree(v)).max().unwrap_or(0)
    }
    fn max_in_degree(&self) -> usize {
        self.vertices().map(|v| self.in_degree(v)).max().unwrap_or(0)
    }
    fn min_out_degree(&self) -> usize {
        self.vertices().map(|v| self.out_degree(v)).min().unwrap_or(0)
    }
    fn min_in_degree(&self) -> usize {
        self.vertices().map(|v| self.in_degree(v)).min().unwrap_or(0)
    }

    fn in_degrees(&self) -> impl Iterator<Item = usize> {
        self.vertices().map(move |v| self.in_degree(v))
    }

    fn out_degrees(&self) -> impl Iterator<Item = usize> {
        self.vertices().map(move |v| self.out_degree(v))
    }

    fn average_out_degree(&self) -> f64 {
        self.size() as f64 / self.order() as f64
    }

    fn average_in_degree(&self) -> f64 {
        self.size() as f64 / self.order() as f64
    }

    /// A graph is symmetric if the `is_edge` relation is symmetric.
    ///
    fn is_symmetric(&self) -> bool {
        self.edges().all(|(src, trg)| self.is_edge(trg, src))
    }

    fn transpose<G: Graph>(&self) -> G {
        G::from_edges(self.edges().map(|(src, trg)| (trg, src)))
    }

    fn symmetric_hull<G: Graph>(&self) -> G {
        G::from_edges(self.edges().flat_map(|(src, trg)| [(src, trg), (trg, src)]))
    }

    /// A graph is (strongly) connected if its transitive closure is the
    /// complete graph of the same order.
    ///
    fn is_strongly_connected(&self) -> bool;

    fn is_weakly_connected(&self) -> bool
    where Self: Sized
    {
        self.symmetric_hull::<Self>().is_strongly_connected()
    }
}

/// The class of undirected graphs.
///
/// An undirected graph is a tuple $(V, E)$ where $V$ is a set of vertices and
/// $E \subseteq {n \choose 2}$ is a *set* of undirected edges. In particular,
/// because edges are sets, an undirected graph does not contain loops where an
/// edge connects a vertex to itself.
///
pub trait UndirectedGraph
    where Self: Graph<Type = Undirected>
{
    fn degree(&self, vertex: usize) -> usize {
        self.adjacents(vertex).count()
    }

    fn max_degree(&self) -> usize {
        self.vertices().map(|v| self.degree(v)).max().unwrap_or(0)
    }

    fn min_degree(&self) -> usize {
        self.vertices().map(|v| self.degree(v)).min().unwrap_or(0)
    }

    fn average_degree(&self) -> f64 {
        self.size() as f64 * 2.0 / self.order() as f64
    }

    fn is_regular(&self) -> bool {
        self.max_degree() == self.min_degree()
    }

    fn is_connected(&self) -> bool;
}

pub fn directed_complete_edge_count(order: usize) -> usize {
    order * order
}

pub fn undirected_complete_edge_count(order: usize) -> usize {
    order * (order - 1) / 2
}
