use crate::DfsEvent;
use crate::DirectedGraph;
use crate::Graph;
use crate::MutableGraph;
use crate::UndirectedGraph;
use crate::random::RandomGraph;
use crate::undirected_complete_edge_count;
use fxhash::FxHashSet;
use rand::Rng;
use rand::RngExt;
use rand::distr::Bernoulli;
use rand::distr::Distribution;
use rand::distr::Uniform;
use std::iter::repeat_with;
use std::vec;
use super::adjacency_set::AdjacencyRep;
use super::directed;

/// Adjacency list representation of an undirected graph.
///
/// Internally, this is just a wrapper around `DirectedGraph` that maintains
/// the additional invariant that the graph is symmetric. This allows to
/// to provide improved performance for some operations.
///
/// # TODO
///
/// In some situations, a more efficient implementation may store two directed
/// graphs, such that one graph the the transpose of the other, i.e. for one
/// graph it would hold for all edges $(i, j)$ that $i < j$ and for the other
/// graph that $i > j$.
///
/// The main benefit would be that iteration over edges would be more efficient
/// because it would not need to skip over half of the edges. There may also be
/// cases where the compiler would be able to drop one half of the graph early,
/// or even skip allocation at all. The main drawback would be that adjacency
/// checks would become more expensive.
///
#[derive(Debug, Clone)]
pub struct AdjacencySets<T = FxHashSet<usize>>(directed::AdjacencySets<T>)
where T: AdjacencyRep + Clone;

impl<T: AdjacencyRep + Clone> UndirectedGraph for AdjacencySets<T> {
    fn degree(&self, vertex: usize) -> usize {
        self.0.out_degree(vertex)
    }

    fn max_degree(&self) -> usize {
        self.0.max_out_degree()
    }

    fn min_degree(&self) -> usize {
        self.0.min_out_degree()
    }

    fn is_regular(&self) -> bool {
        self.min_degree() == self.max_degree()
    }

    /// Cost: O(V + E)
    ///
    fn is_connected(&self) -> bool {
        let mut visited = vec![false; self.order()];
        let mut stack = vec![0];
        visited[0] = true;

        while let Some(vertex) = stack.pop() {
            for neighbor in self.0.adjacents(vertex) {
                if !visited[neighbor] {
                    visited[neighbor] = true;
                    stack.push(neighbor);
                }
            }
        }
        let result =visited.into_iter().all(|v| v);
        debug_assert_eq!(result,
            self.0.is_strongly_connected(),
            "Undirected graph are connected if and only if the underlying directed graph is strongly connected"
        );
        result
    }
}

impl<T: AdjacencyRep + Clone> Graph for AdjacencySets<T> {

    fn empty(order: usize) -> Self {
        AdjacencySets(directed::AdjacencySets::<T>::empty(order))
    }

    fn from_edges(edges: impl IntoIterator<Item = (usize, usize)>) -> Self {
        Self::from_edges_with_order(0, edges)
    }

    fn from_edges_with_order(order: usize, edges: impl IntoIterator<Item = (usize, usize)>) -> Self {
        let undirected = edges
            .into_iter()
            .flat_map(|(src, trg)| [(src, trg), (trg, src)]);
        AdjacencySets(directed::AdjacencySets::from_edges_with_order(order, undirected))
    }

    fn from_adjacency_lists(adjacency_lists: impl IntoIterator<Item = impl IntoIterator<Item = usize>>) -> Self {
        let iter = adjacency_lists.into_iter();
        let (lower, upper) = iter.size_hint();
        let order = upper.unwrap_or(lower);
        Self::from_edges_with_order(
            order,
            iter
                .enumerate()
                .flat_map(|(src, adjs)| {
                    adjs.into_iter().map(move |trg| (src, trg))
                })
        )
    }

    fn order(&self) -> usize {
        self.0.order()
    }

    fn size(&self) -> usize {
        self.0.size() / 2
    }

    fn vertices(&self) -> impl Iterator<Item = usize> {
        self.0.vertices()
    }

    fn edges(&self) -> impl Iterator<Item = (usize, usize)> {
        self.0.edges().filter(|(i, j)| i < j)
    }

    fn adjacents(&self, vertex: usize) -> impl Iterator<Item = usize> {
        self.0.adjacents(vertex)
    }

    fn is_edge(&self, source: usize, target: usize) -> bool {
        debug_assert_eq!(self.0.is_edge(source, target), self.0.is_edge(target, source));
        self.0.is_edge(source, target)
    }

    /// FIXME: adjust the hook so that we enter an undirected edge only once?
    fn dfs<F: FnMut(DfsEvent) -> bool>(&self, vertex: usize, hook: F) {
        self.0.dfs(vertex, hook)
    }

    fn is_complete(&self) -> bool {
        // graph::DirectedMutableGraph::is_complete(&self.0)
        self.0.is_complete()
    }

    fn is_reachable(&self, source: usize, target: usize) -> bool {
        self.0.is_reachable(source, target)
    }

    fn complete_edge_count(&self) -> usize {
        undirected_complete_edge_count(self.order())
    }
}

impl<T> RandomGraph for AdjacencySets<T>
where
    T: AdjacencyRep + Clone,
{
    /// Random graph in the $G_{n,p}$ model. Each edge is included with
    /// probability $p$. For $p = \frac{1}{2}$, this is the uniform distribution
    /// over undirected graphs of order $n$.
    ///
    fn gnp_with_rng<R: Rng>(order: usize, p: f64, mut rng: R) -> Self
    {
        let d = Bernoulli::new(p).unwrap();
        let edges = (0..order)
            .flat_map(move |i|
                (i+1..order).map(move |j| (i, j))
            )
            .filter(move |_| d.sample(&mut rng));
        Self::from_edges_with_order(order, edges)
    }

    /// Random graph in the $G_{n,m}$ model. This is the uniform distribution
    /// over undirected graphs of order $n$ and size $m$.
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
        let mut seen = FxHashSet::<(usize, usize)>::default();
        seen.reserve(m);
        let d = Uniform::new(0, order).unwrap();
        let edges =
            repeat_with(move || {
                let i = d.sample(&mut rng);
                let j = d.sample(&mut rng);
                if i < j { (i, j) } else { (j, i) }
            })
            .filter(move |&(i, j)| i != j && seen.insert((i, j)))
            .take(m);
        Self::from_edges_with_order(order, edges)
    }

    /// Uniformly distributed random directed regular graphs.
    ///
    /// This function panics if the parameters do not allow for a regular graph,
    /// i.e. if both the degree and the order are odd.
    ///
    /// # Notes
    ///
    /// This algorithm is not complete. It can fail, in which case it should be
    /// retried with a different seed.
    ///
    /// The graph is sampled using the "configuration model" that was first
    /// described by Bollobás in "A probabilistic proof of an asymptotic formula
    /// for the number of labelled regular graphs" (1980).  The goal of the
    /// method as described in the paper was not to efficiently sample random
    /// regular graphs, but count the number of those graphs.  In its original
    /// form the algorithm succeeds in each iteration with a probability of
    /// $\exp(-\frac{(d-1)^2}{4})$ where $d$ is the degree of the graph.
    ///
    /// More efficient algorithms exists for up to roughly $d = O(\sqrt{n})$.
    /// There are also algorithms that trade efficiency for the quality of the
    /// distribution such that the results are only asymptotically uniform.
    ///
    /// The best know exactly uniform algorithms that support a wide range of
    /// $d$ values are based on the "switching method", where a pair is created
    /// in the configuration model first. Loops and parallel edges are then
    /// removed by performing a sequence of "switches" while maintaining
    /// uniformity of the sampling by rejecting switches based on the bias that
    /// they would introduce.
    ///
    fn rrg_with_rng<R: Rng>(order: usize, degree: usize, attempts: u64, mut rng: R) -> Self
    where
    {
        if !(order * degree).is_multiple_of(2) {
            panic!("A regular graph of odd degree cannot have an odd number of vertices.");
        }

        for _ in 0..attempts {
            if let Some(g) = try_rrg_with_rng(order, degree, &mut rng) {
                return g;
            }
        }
        panic!("Failed to generate a random regular graph after {} attempts. Consider increasing the number of attempts or using a different seed.", attempts);
    }
}

/// This is the original configuration model from Bollobás (1980).
///
pub fn random_pairing<R>(
    order: usize,
    degree: usize,
    mut rng: R
) -> impl IntoIterator<Item = (usize, usize)>
where
    R: Rng,
{
    let mut stubs = Vec::with_capacity(order * degree);
    let l = order * degree;
    for i in 0..order {
        for _ in 0..degree {
            stubs.push(i);
        }
    }
    // randomly partition the stubs into two equally sized sets and uniformly
    // shuffle the second set (the first set is not shuffled).
    (0..l / 2).map(move |i| {
        let j = rng.random_range(i*2..l);
        let src = stubs[i*2];
        let trg = stubs[j];
        stubs.swap(i*2+1, j);
        if src < trg { (src, trg) } else { (trg, src) }
    })
}

fn try_rrg_with_rng<T, R>(
    order: usize,
    degree: usize,
    rng: R
) -> Option<AdjacencySets<T>>
where
    T: AdjacencyRep + Clone,
    R: Rng
{
    let mut g = AdjacencySets::<T>::empty(order);
    for (i, j) in random_pairing(order, degree, rng) {
        if !g.add_edge(i, j) {
            return None;
        }
    }
    Some(g)
}

impl<T: AdjacencyRep + Clone> MutableGraph for AdjacencySets<T> {
    fn add_edge(&mut self, source: usize, target: usize) -> bool{
        self.0.add_edge(source, target);
        self.0.add_edge(target, source)
    }

    fn remove_edge(&mut self, source: usize, target: usize) -> bool{
        self.0.remove_edge(source, target);
        self.0.remove_edge(target, source)
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::*;
    use divan::{Bencher, black_box};
    use rand::SeedableRng;

    const ORDERS: &[usize; 4] = &[100, 1000, 10000, 100000];

    mod radom_pairing {
        use super::*;

        #[divan::bench(
            args = ORDERS,
        )]
        fn bench_random_pairing(b: Bencher, order: usize) {
            b
            .counter(order)
            .with_inputs(|| rand_pcg::Pcg64::seed_from_u64(0))
            .bench_local_values(|mut rng| {
                let r: Vec<(usize, usize)> = random_pairing(order, 10, &mut rng).into_iter().collect();
                black_box(r)
            });
        }
    }

    mod try_rrg_with_rng {
        use super::*;

        #[divan::bench(
            args = ORDERS,
        )]
        fn bench_try_rrg_with_rng(b: Bencher, order: usize) {
            b
            .counter(order)
            .with_inputs(|| rand_pcg::Pcg64::seed_from_u64(0))
            .bench_local_values(|mut rng| {
                let r = try_rrg_with_rng::<FxHashSet<usize>, _>(order, 10, &mut rng);
                black_box(r)
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::named::PETERSEN_GRAPH;

    use super::*;

    #[test]
    fn petersen_graph() {
        let g: AdjacencySets<FxHashSet<usize>> = AdjacencySets::from_adjacency_lists(PETERSEN_GRAPH);
        assert_eq!(g.order(), 10);
        assert_eq!(g.size(), 15);
        assert!(g.is_connected());
        assert_eq!(g.min_degree(), 3);
        assert_eq!(g.max_degree(), 3);
        assert!(g.is_regular());
        assert_eq!(g.degree(0), 3);
    }

    #[test]
    fn test_gnp() {
        let order = 100;
        let p = 0.5;
        // expected number of edges:
        let m_expected = order as f64 * (order as f64 - 1.0) / 2.0 * p;
        let m_variance = order as f64 * (order as f64 - 1.0) / 2.0 * p * (1.0 - p);
        // z-score for 0.999999 quantile
        let m_zscore = 4.753424;
        let m_quantile_upper = m_expected + m_zscore * m_variance.sqrt();
        let m_quantile_lower = m_expected - m_zscore * m_variance.sqrt();


        let g: AdjacencySets<FxHashSet<usize>> = AdjacencySets::gnp(order, p);
        assert_eq!(g.order(), order);
        println!("Gnp: G_{{{}, {}}}, size {}, expected size {}, quantile lower {}, quantile upper {}", order, p, g.size(), m_expected, m_quantile_lower, m_quantile_upper);
        println!("Gnp: G_{{{}, {}}} max_degree {}, min_degree {}", order, p, g.max_degree(), g.min_degree());
        assert!(g.size() <= m_quantile_upper as usize, "Size {} is larger than the 0.999999 quantile {} for G(n, p) with n = {} and p = {}", g.size(), m_quantile_upper, order, p);
        assert!(g.size() >= m_quantile_lower as usize, "Size {} is smaller than the 0.000001 quantile {} for G(n, p) with n = {} and p = {}", g.size(), m_quantile_lower, order, p);
        assert!(g.is_connected());
    }

    #[test]
    fn test_gnm() {
        let order = 100;
        let m = 500;
        let g: AdjacencySets<FxHashSet<usize>> = AdjacencySets::gnm(order, m);
        assert_eq!(g.order(), order);
        assert_eq!(g.size(), m);
        println!("Gnm: G_{{{}, {}}} Size: {}, max_degree {}, min_degree {}", order, m, g.size(), g.max_degree(), g.min_degree());
        assert!(g.is_connected());
    }

    #[test]
    fn test_rrg() {
        let order = 1000;
        let degree = 5;
        let attempts = 10000;
        let g: AdjacencySets<FxHashSet<usize>> = AdjacencySets::rrg(order, degree, attempts);
        assert_eq!(g.order(), order);
        assert_eq!(g.min_degree(), degree);
        assert_eq!(g.max_degree(), degree);
        println!("RRG: G_{{{}, {}}} Size: {}, max_degree {}, min_degree {}", order, degree, g.size(), g.max_degree(), g.min_degree());
        assert!(g.is_connected());
        assert!(g.is_regular());
    }
}