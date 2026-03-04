//! Adjacency list representations for sparse graphs.
//!
//! -   If in doubt, use `FxHashSet`.
//! -   If the graph needs to be updated frequently, use `FxHashSet`.
//! -   If the graph is mostly static and has a maximum degree of less than
//!     about 300, use `SortedVector`.
//! -   If edges adjacent to a vertex are only iterated over and almost
//!     never looked up, use `Vec`.
//! -   Use `FxHashSet` for everything else.
//!

use std::{hash::BuildHasher};

pub trait AdjacencyRep
where
    Self: PartialEq + Eq + Clone,
    Self: IntoIterator<Item = usize> + FromIterator<usize>,
{
    type Iter<'a>: Iterator<Item = usize> where Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;

    fn empty_set() -> Self;

    fn degree(&self) -> usize;
    fn is_adjacent(&self, vertex: usize) -> bool;
    fn add_adjacent(&mut self, vertex: usize) -> bool;
    fn add_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>);

    fn remove_adjacent(&mut self, vertex: usize) -> bool;

    /// Implementations should optimize for the case when the number of removed
    /// vertices is large, i.e. in the order of the overall number of adjacent
    /// vertices.
    ///
    /// Cost: $O(n)$ where $n$ is the number of adjacent vertices.
    ///
    fn remove_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>);
}

/// Adjacency list representation via unsorted vectors.
///
/// This representation is suitable only for algorithms that mostly perform 
/// unordered enumeration of adjacent vertices and do not require frequent
/// adjacency checks or updates.  It supports very fast graph construction. It
/// also is very memory efficient and has good cache locality.
///
/// # IMPORTANT
///
/// It is the responsibility of the user to ensure that no duplicate edges
/// are added to the graph.
///
/// # FIXME
///
/// This implementation currently violates the requirements for equality.
///
impl AdjacencyRep for Vec<usize> {
    type Iter<'a> = std::vec::IntoIter<usize> where Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.clone().into_iter()
    }

    #[inline]
    fn empty_set() -> Self {
        Vec::new()
    }

    #[inline]
    fn degree(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_adjacent(&self, vertex: usize) -> bool {
        self.contains(&vertex)
    }

    // TODO this can create duplicates. If we ignore them here we have to
    // take care of them during removal.
    #[inline]
    fn add_adjacent(&mut self, vertex: usize) -> bool {
        self.push(vertex);
        true
    }

    #[inline]
    fn add_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        self.extend(vertices);
    }

    #[inline]
    fn remove_adjacent(&mut self, vertex: usize) -> bool {
        if let Some(pos) = self.iter().position(|x| x == vertex) {
            self.swap_remove(pos);
            true
        } else {
            false
        }
    }

    /// Note that this is optimized for only a very small number of vertices to
    /// be removed. The reason is that only for graphs with very small maximum
    /// degree the use of `Vec` makes sense. If you find yourself removing a
    /// larger number of vertices from an adjacency list, you should probably
    /// use a different representation in first place.
    ///
    /// Cost: $O(1)$
    ///
    #[inline]
    fn remove_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        for vertex in vertices {
            self.remove_adjacent(vertex);
        }
    }
}

impl<H: BuildHasher + Default + Clone> AdjacencyRep for std::collections::HashSet<usize, H> {
    type Iter<'a> = std::iter::Copied<std::collections::hash_set::Iter<'a, usize>>
       where Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }

    #[inline]
    fn empty_set() -> Self {
        std::collections::HashSet::with_hasher(Default::default())
    }

    #[inline]
    fn degree(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_adjacent(&self, vertex: usize) -> bool {
        self.contains(&vertex)
    }

    #[inline]
    fn add_adjacent(&mut self, vertex: usize) -> bool {
        self.insert(vertex)
    }

    #[inline]
    fn add_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        self.extend(vertices);
    }

    #[inline]
    fn remove_adjacent(&mut self, vertex: usize) -> bool {
        self.remove(&vertex)
    }

    #[inline]
    fn remove_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        let to_remove: std::collections::HashSet<usize, H> = vertices.into_iter().collect();
        self.retain(|v| !to_remove.contains(v));
    }
}

impl AdjacencyRep for std::collections::BTreeSet<usize> {
    type Iter<'a> = std::iter::Copied<std::collections::btree_set::Iter<'a, usize>>
       where Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.iter().copied()
    }

    #[inline]
    fn empty_set() -> Self {
        std::collections::BTreeSet::new()
    }

    #[inline]
    fn degree(&self) -> usize {
        self.len()
    }

    #[inline]
    fn is_adjacent(&self, vertex: usize) -> bool {
        self.contains(&vertex)
    }

    #[inline]
    fn add_adjacent(&mut self, vertex: usize) -> bool {
        self.insert(vertex)
    }

    #[inline]
    fn add_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        self.extend(vertices);
    }

    #[inline]
    fn remove_adjacent(&mut self, vertex: usize) -> bool {
        self.remove(&vertex)
    }

    #[inline]
    fn remove_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        let to_remove: std::collections::BTreeSet<usize> = vertices.into_iter().collect();
        self.retain(|v| !to_remove.contains(v));
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SortedVector(Vec<usize>);

impl IntoIterator for SortedVector {
    type Item = usize;
    type IntoIter = std::vec::IntoIter<usize>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<usize> for SortedVector {
    #[inline]
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = usize>,
    {
        let mut vec = iter.into_iter().collect::<Vec<_>>();
        vec.sort_unstable();
        vec.dedup();
        SortedVector(vec)
    }
}

impl AdjacencyRep for SortedVector {
    type Iter<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>
    where Self: 'a;

    #[inline]
    fn iter(&self) -> Self::Iter<'_> {
        self.0.as_slice().iter().copied()
    }

    #[inline]
    fn empty_set() -> Self {
        SortedVector(Vec::new())
    }

    #[inline]
    fn degree(&self) -> usize {
        self.0.len()
    }

    #[inline]
    fn is_adjacent(&self, vertex: usize) -> bool {
        self.0.binary_search(&vertex).is_ok()
    }

    #[inline]
    fn add_adjacent(&mut self, vertex: usize) -> bool {
        if let Err(pos) = self.0.binary_search(&vertex) {
            self.0.insert(pos, vertex);
            true
        } else {
            false
        }
    }

    /// # TODO
    /// define the thresholds as constants and use benchmarks to tune them.
    ///
    /// It is not clear whether repeated shifting is that much cheaper than
    /// sorting. It would be nice if there would be an insert_many function.
    ///
    #[inline]
    fn add_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        let vertices = vertices.into_iter();
        let (_lower, upper) = vertices.size_hint();
        if let Some(upper) = upper && upper < 10 && self.0.len() > 100 {
            for vertex in vertices {
                self.add_adjacent(vertex);
            }
        } else {
            self.0.extend(vertices);
            self.0.sort_unstable();
            self.0.dedup();
        }
    }

    #[inline]
    fn remove_adjacent(&mut self, vertex: usize) -> bool {
        if let Ok(pos) = self.0.binary_search(&vertex) {
            self.0.remove(pos);
            true
        } else {
            false
        }
    }

    #[inline]
    fn remove_adjacents(&mut self, vertices: impl IntoIterator<Item = usize>) {
        // sort the inputs
        let mut to_remove = vertices.into_iter().collect::<Vec<_>>();
        to_remove.sort_unstable();

        // iterate through the adjacency list and the list of vertices to be removed.
        let mut i = to_remove.into_iter().peekable();
        self.0.retain(|v| {
            while i.peek().is_some_and(|r| r < v) {
                i.next();
            }
            if i.peek().is_some_and(|r| r == v) {
                i.next();
                false // Remove
            } else {
                true  // Keep
            }
        });
    }
}

#[cfg(feature = "bench")]
mod benches {
    use super::*;
    use divan::{Bencher, black_box};
    use std::collections::HashSet;
    use std::collections::BTreeSet;
    use fxhash::FxHashSet;

    const SIZES: &[usize] = &[1, 2, 10, 20, 100, 200, 500, 1000, 2000, 5000, 10000];

    /// Benchmarks for linear input lists
    ///
    #[divan::bench_group(name = "linear")]
    mod linear {
        use super::*;
        use rand::RngExt;

        fn make_empty_inputs<T: AdjacencyRep>(i: usize) -> (T, impl IntoIterator<Item = usize>) {
            let adj = T::empty_set();
            (adj, 0..i)
        }

        fn make_filled_inputs<T: AdjacencyRep>(i: usize) -> (T, Vec<usize>) {
            let (mut adj, vertices) = make_empty_inputs::<T>(i);
            adj.add_adjacents(vertices);
            let rng = rand::rng();
            let vertices: Vec<usize> = rng
                .random_iter()
                .take(i)
                .map(|v: u64| (v % i as u64) as usize)
                .collect();
            (adj, vertices)
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_add_adjacent<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_empty_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices)| {
                for i in vertices {
                    adj.add_adjacent(i);
                }
                divan::black_box_drop(adj);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_add_adjacents<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices)| {
                adj.add_adjacents(vertices);
                divan::black_box_drop(adj);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_is_adjacent<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(adj, vertices): (T, Vec<usize>)| {
                let mut r = true;
                for i in vertices {
                    r &= adj.is_adjacent(black_box(i));
                }
                divan::black_box_drop(r);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_remove_adjacent<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices): (T, Vec<usize>)| {
                for i in vertices {
                    adj.remove_adjacent(black_box(i));
                }
                divan::black_box_drop(adj);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_remove_adjacents<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices): (T, Vec<usize>)| {
                adj.remove_adjacents(vertices);
                divan::black_box_drop(adj);
            });
        }
    }

    /// Benchmarks for random input lists
    ///
    mod random {
        use super::*;
        use rand::RngExt;

        fn make_empty_inputs<T: AdjacencyRep>(i: usize) -> (T, Vec<usize>) {
            let adj = T::empty_set();
            let rng = rand::rng();
            let vertices: Vec<usize> = rng
                .random_iter()
                .take(i)
                .map(|v: u64| (v % i as u64) as usize)
                .collect();
            (adj, vertices)
        }

        fn make_filled_inputs<T: AdjacencyRep>(i: usize) -> (T, Vec<usize>) {
            let (mut adj, vertices) = make_empty_inputs::<T>(i);
            adj.add_adjacents(vertices);
            let rng = rand::rng();
            let vertices: Vec<usize> = rng
                .random_iter()
                .take(i)
                .map(|v: u64| (v % i as u64) as usize)
                .collect();
            (adj, vertices)
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, FxHashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_add_adjacent<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_empty_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices): (T, Vec<usize>)| {
                for i in vertices {
                    adj.add_adjacent(i);
                }
                divan::black_box_drop(adj);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, FxHashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_add_adjacents<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices): (T, Vec<usize>)| {
                adj.add_adjacents(vertices);
                divan::black_box_drop(adj);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, FxHashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_is_adjacent<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(adj, vertices): (T, Vec<usize>)| {
                let mut r = true;
                for i in vertices {
                    r &= adj.is_adjacent(i);
                }
                divan::black_box_drop(r);
            });
        }

        #[divan::bench(
            // types = [SortedVector, HashSet<usize>, FxHashSet<usize>, BTreeSet<usize>],
            types = [SortedVector, HashSet<usize>, FxHashSet<usize>],
            args = &[1_000_000, 5_000_000, 10_000_000, 50_000_000, 100_000_000],
            sample_count = 5,
            sample_size = 1,
        )]
        fn bench_is_adjacent_large<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_refs(|args| {
                let mut r = true;
                for i in black_box(args.1.iter().take(5)) {
                    r &= args.0.is_adjacent(i);
                }
                // divan::black_box_drop(args);
                divan::black_box_drop(r);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, FxHashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_remove_adjacent<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices): (T, Vec<usize>)| {
                for i in vertices {
                    adj.remove_adjacent(i);
                }
                divan::black_box_drop(adj);
            });
        }

        #[divan::bench(
            types = [SortedVector, Vec<usize>, HashSet<usize>, FxHashSet<usize>, BTreeSet<usize>],
            args = SIZES
        )]
        fn bench_remove_adjacents<T: AdjacencyRep>(b: Bencher, n: usize) {
            b
            .counter(n)
            .with_inputs(|| make_filled_inputs::<T>(n))
            .bench_local_values(|(mut adj, vertices): (T, Vec<usize>)| {
                adj.remove_adjacents(vertices);
                divan::black_box_drop(adj);
            });
        }
    }
}