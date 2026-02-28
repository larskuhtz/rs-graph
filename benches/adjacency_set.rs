#[allow(unused_imports)]
use graph::sparse::adjacency_set;

use divan::AllocProfiler;

#[global_allocator]
static ALLOC: AllocProfiler = AllocProfiler::system();

fn main() {
    // Run registered benchmarks.
    divan::main();
}