use std::iter::repeat_with;

use super::adjacency_set::AdjacencyRep;
use crate::DirectedGraph;
use crate::DirectedMutableGraph;
use crate::Graph;
use crate::DfsEvent;
use crate::MutableGraph;
use crate::directed_complete_edge_count;
use crate::random::RandomGraph;
use fxhash::FxHashSet;
use rand::Rng;
use rand::RngExt;
use rand::distr::Bernoulli;
use rand::distr::Distribution;
use rand::distr::Uniform;

/// Adjacency list representation of a directed graph.
///
/// The data structure and associated functions are optimized for sparse graphs.
///
/// In most cases the use of `FxHashSet` is recommended for the adjacency set
/// representation. On large graphs it can be beneficial to swich the
/// representation based on algorithmic needs.
///
#[derive(Debug, Clone)]
pub struct AdjacencySets<T = FxHashSet<usize>> (Vec<T>)
where T: AdjacencyRep + Clone;

impl<T: AdjacencyRep + Clone> FromIterator<(usize, usize)> for AdjacencySets<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (usize, usize)>>(iter: I) -> Self {
        Self::from_edges(iter)
    }
}

impl<T: AdjacencyRep + Clone> FromIterator<T> for AdjacencySets<T> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::from_adjacency_lists(iter)
    }
}

impl<T: AdjacencyRep + Clone> Graph for AdjacencySets<T> {

    fn empty(order: usize) -> Self {
        AdjacencySets(vec![T::empty_set(); order])
    }

    fn from_edges(edges: impl IntoIterator<Item = (usize, usize)>) -> Self {
        Self::from_edges_with_order(0, edges)
    }

    fn from_edges_with_order(order: usize, edges: impl IntoIterator<Item = (usize, usize)>) -> Self {
        let mut neighbors = vec![T::empty_set(); order];
        for (source, target) in edges {
            if source >= neighbors.len() {
                neighbors.resize(source + 1, T::empty_set());
            }
            if target >= neighbors.len() {
                neighbors.resize(target + 1, T::empty_set());
            }
            neighbors[source].add_adjacent(target);
        }
        AdjacencySets(neighbors)
    }

    fn from_adjacency_lists(adjacency_lists: impl IntoIterator<Item = impl IntoIterator<Item = usize>>) -> Self {
        let adjs = adjacency_lists
            .into_iter()
            .map(T::from_iter)
            .collect::<Vec<_>>();
        AdjacencySets(adjs)
    }

    fn order(&self) -> usize {
        self.0.len()
    }

    fn size(&self) -> usize {
        self.0.iter().map(|n| n.degree()).sum::<usize>()
    }

    fn vertices(&self) -> impl Iterator<Item = usize> {
        0..self.order()
    }

    fn edges(&self) -> impl Iterator<Item = (usize, usize)> {
        self.0.iter().enumerate().flat_map(move |(i, adjs)| {
            adjs.iter().map(move |j| (i, j))
        })
    }

    fn adjacents(&self, vertex: usize) -> impl Iterator<Item = usize> {
        self.0[vertex].iter()
    }

    fn is_edge(&self, source: usize, target: usize) -> bool {
        self.0[source].is_adjacent(target)
    }

    fn dfs<F: FnMut(DfsEvent) -> bool>(&self, vertex: usize, mut hook: F)
    {
        let mut discovered = vec![false; self.order()];
        let mut visited = vec![false; self.order()];

        // Initialize Stack
        let mut stack = vec![vertex];
        discovered[vertex] = true;

        while let Some(current) = stack.last() {
            let current = *current;
            if !visited[current] {
                hook(DfsEvent::Pre(current));
                visited[current] = true;
                for neighbor in self.0[current].iter() {
                    let explore = hook(DfsEvent::Edge(current, neighbor));
                    if explore && !discovered[neighbor] {
                        stack.push(neighbor);
                        discovered[neighbor] = true;
                    }
                }
            } else {
                hook(DfsEvent::Post(current));
                stack.pop();
            }
        }
    }

    // FIXME: this is pretty inefficient. At least we should implement
    // early termination for DFS.
    fn is_reachable(&self, source: usize, target: usize) -> bool {
        let mut reachable = false;
        self.dfs(source, |e| {
            if let DfsEvent::Pre(v) = e && v == target {
                reachable = true;
                return false; // Prune search
            }
            true
        });
        reachable
    }

    fn complete_edge_count(&self) -> usize {
        directed_complete_edge_count(self.order())
    }
}

impl<T: AdjacencyRep + Clone> MutableGraph for AdjacencySets<T> {
    fn add_edge(&mut self, source: usize, target: usize) -> bool {
        self.0[source].add_adjacent(target)
    }

    fn remove_edge(&mut self, source: usize, target: usize) -> bool {
        self.0[source].remove_adjacent(target)
    }
}

impl<T: AdjacencyRep + Clone> DirectedMutableGraph for AdjacencySets<T> {
    fn symmetric_hull_mut(&mut self) {
        for i in 0..self.order() {
            for j in 0..self.0[i].degree() {
                self.0[j].add_adjacent(i);
            }
        }
    }
}

impl<T> DirectedGraph for AdjacencySets<T>
where T: AdjacencyRep + Clone,
{
    fn out_degree(&self, vertex: usize) -> usize {
        self.0[vertex].degree()
    }

    fn in_degree(&self, vertex: usize) -> usize {
        self.0.iter().filter(|neighbors| neighbors.is_adjacent(vertex)).count()
    }

    fn max_out_degree(&self) -> usize {
        self.0.iter().map(|neighbors| neighbors.degree()).max().unwrap_or(0)
    }

    fn max_in_degree(&self) -> usize {
        let mut max_in_degree = 0;
        for vertex in self.vertices() {
            let in_degree = self.in_degree(vertex);
            if in_degree > max_in_degree {
                max_in_degree = in_degree;
            }
        }
        max_in_degree
    }

    fn min_out_degree(&self) -> usize {
        self.0.iter().map(|neighbors| neighbors.degree()).min().unwrap_or(0)
    }

    fn min_in_degree(&self) -> usize {
        let mut min_in_degree = usize::MAX;
        for vertex in self.vertices() {
            let in_degree = self.in_degree(vertex);
            if in_degree < min_in_degree {
                min_in_degree = in_degree;
            }
        }
        min_in_degree
    }

    fn is_symmetric(&self) -> bool {
        for (i, neighbors) in self.0.iter().enumerate() {
            for a in neighbors.iter() {
                if !self.0[a].is_adjacent(i) {
                    return false;
                }
            }
        }
        true
    }

    fn transpose<G: Graph>(&self) -> G
    {
        G::from_edges_with_order(
            self.order(),
            self.edges().map(|(source, target)| (target, source))
        )
    }

    fn symmetric_hull<G: Graph>(&self) -> G {
        G::from_edges_with_order(
            self.order(),
            self.edges().flat_map(|(src, trg)|
                [(src, trg), (trg, src)]
            ),
        )
    }

    /// Check if graph is strongly connected
    ///
    fn is_strongly_connected(&self) -> bool {
        let sccs = self.sccs_(0, &mut vec![false; self.order()]);
        sccs.len() == 1 && sccs[0].len() == self.order()
    }
}

impl<T> RandomGraph for AdjacencySets<T>
where
    T: AdjacencyRep + Clone,
{

    /// Random graph in the $G_{n,p}$ model. Each edge is included with
    /// probability $p$. For $p = \frac{1}{2}$, this is the uniform distribution
    /// over all directed graphs of order $n$.
    ///
    fn gnp_with_rng<R: Rng>(order: usize, p: f64, mut rng: R) -> Self
    {
        let d = Bernoulli::new(p).unwrap();
        let vertices = (0..order).map(|_| d
            .sample_iter(&mut rng)
            .take(order)
            .enumerate()
            .filter(|&(_, edge)| edge)
            .map(|(j, _)| j)
            .collect()
        ).collect::<Vec<_>>();
        AdjacencySets(vertices)
    }

    /// Random graph in the $G_{n,m}$ model. This is the uniform distribution
    /// over all directed graphs of order $n$ and size $m$.
    ///
    /// # Notes
    ///
    /// This implementation is efficient only for sparse graphs. For almost
    /// complete graphs this algorithm becomes infeasible.
    ///
    /// TODO: For dense graphs use the rand::RandomIterator::sample
    ///
    fn gnm_with_rng<R: Rng>(order: usize, m: usize, mut rng: R) -> Self
    {
        let d = Uniform::new(0, order).unwrap();
        let mut seen = FxHashSet::<(usize, usize)>::default();
        seen.reserve(m);

        let edges =
            repeat_with(move || (d.sample(&mut rng), d.sample(&mut rng)))
            .filter(move |&(i, j)| seen.insert((i, j)))
            .take(m);
        Self::from_edges_with_order(order, edges)
    }

    /// Uniformly distributed random directed regular graphs.
    ///
    /// For directed regular graphs the attempts paramter is ignored.
    ///
    fn rrg_with_rng<R: Rng>(order: usize, degree: usize, _attempts: u64, mut rng: R) -> Self
    {
        let mut stubs: Vec<usize> = (0..order).collect();
        let vertices = (0..order).map(|_| {
            (0..degree).map(|i| {
                let trg = rng.random_range(i+1..order);
                stubs.swap(i, trg);
                stubs[i]
            }).collect()
        }).collect();
        AdjacencySets(vertices)
    }
}

impl<T: AdjacencyRep + Clone> AdjacencySets<T> {

    /// An implementation of Tarjan's algorithm for finding strongly connected
    /// components (SCCs).
    ///
    pub fn sccs_(&self, vertex: usize, visited: &mut [bool]) -> Vec<Vec<usize>> {
        let mut sccs = Vec::new();
        let mut index = vec![0; self.order()];
        let mut lowlink: Vec<usize> = vec![usize::MAX; self.order()];
        let mut scc_stack = Vec::new();
        let mut current_index = 0;

        self.dfs(
            vertex,
            |e| match e {
                DfsEvent::Edge(_, target) => {
                    // This is the edge hook, we don't need to do anything here for Tarjan's algorithm
                    !visited[target]
                },
                DfsEvent::Pre(v) => {
                    // This is the pre-order hook, we initialize the index and lowlink values here
                    index[v] = current_index;
                    lowlink[v] = current_index;
                    visited[v] = true;
                    current_index += 1;
                    scc_stack.push(v);
                    true
                },
                DfsEvent::Post(v) => {
                    // This is the post-order hook, we update the lowlink values and check for SCCs here
                    for neighbor in self.0[v].iter() {
                        // Note that uninitialized lowlink values are set to usize::MAX
                        lowlink[v] = lowlink[v].min(lowlink[neighbor]);
                    }
                    if lowlink[v] == index[v] {
                        let mut scc = Vec::new();
                        loop {
                            let w = scc_stack.pop().unwrap();
                            lowlink[w] = usize::MAX;
                            scc.push(w);
                            if w == v { break; }
                        }
                        sccs.push(scc);
                    }
                    true
                },
            }
        );
        sccs
    }

    pub fn sccs(&self) -> Vec<Vec<usize>> {
        let mut visited = vec![false; self.order()];
        let mut sccs = Vec::new();
        for vertex in self.vertices() {
            if !visited[vertex] {
                let mut sccs_ = self.sccs_(vertex, &mut visited);
                sccs.append(&mut sccs_);
            }
        }
        sccs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fxhash::FxHashSet;

    #[test]
    fn test_sccs() {
        let graph: AdjacencySets<FxHashSet<usize>> = AdjacencySets::from_edges_with_order(5, [
            (0, 1), (1, 2), (2, 0), // SCC 1
            (3, 4), (4, 3), // SCC 2
            (2, 3) // Edge from SCC 1 to SCC 2
        ]);
        let sccs = graph.sccs();
        assert_eq!(sccs.len(), 2);
        assert!(sccs.iter().any(|scc| scc.contains(&0) && scc.contains(&1) && scc.contains(&2)));
        assert!(sccs.iter().any(|scc| scc.contains(&3) && scc.contains(&4)));
    }
}