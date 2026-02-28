use rand::{Rng, SeedableRng};
use rand_pcg::Pcg64;

use crate::Graph;

pub trait RandomGraph: Graph
{
    fn gnp_with_rng<R: Rng>(order: usize, p: f64, rng: R) -> Self;
    fn gnm_with_rng<R: Rng>(order: usize, size: usize, rng: R) -> Self;
    fn rrg_with_rng<R: Rng>(order: usize, degree: usize, attempts: u64, rng: R) -> Self;

    fn gnp(order: usize, p: f64) -> Self
    where Self: Sized,
    {
        Self::gnp_with_rng(order, p, Pcg64::from_rng(&mut rand::rng()))
    }

    fn gnm(order: usize, size: usize) -> Self
    where Self: Sized,
    {
        Self::gnm_with_rng(order, size, Pcg64::from_rng(&mut rand::rng()))
    }

    fn rrg(order: usize, degree: usize, attempts: u64) -> Self
    where Self: Sized,
    {
        Self::rrg_with_rng(order, degree, attempts, Pcg64::from_rng(&mut rand::rng()))
    }

    fn gnp_with_seed(order: usize, p: f64, seed: u64) -> Self
    where Self: Sized,
    {
        Self::gnp_with_rng(order, p, Pcg64::seed_from_u64(seed))
    }

    fn gnm_with_seed(order: usize, size: usize, seed: u64) -> Self
    where Self: Sized,
    {
        Self::gnm_with_rng(order, size, Pcg64::seed_from_u64(seed))
    }

    fn rrg_with_seed(order: usize, degree: usize, attempts: u64, seed: u64) -> Self
    where Self: Sized,
    {
        Self::rrg_with_rng(order, degree, attempts, Pcg64::seed_from_u64(seed))
    }
}
