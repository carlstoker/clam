//! Algorithms for K Nearest Neighbor search.
//!
//! The stable algorithms are `Linear` and `RepeatedRnn`, with the default being
//! `RepeatedRnn`.
//!
//! We will experiment with other algorithms in the future, and they will be added
//! to this enum as they are being implemented. They should not be considered
//! stable until they are documented as such.

use distances::Number;

use crate::{Dataset, Tree};

pub(crate) mod expanding_threshold;
pub(crate) mod linear;
pub(crate) mod repeated_rnn;
pub(crate) mod sieve_v1;
pub(crate) mod sieve_v2;

/// The algorithm to use for K-Nearest Neighbor search.
///
/// The default is `RepeatedRnn`, as determined by the benchmarks in the crate.
#[derive(Clone, Copy, Debug)]
pub enum Algorithm {
    /// Use linear search on the entire dataset.
    ///
    /// This is a stable algorithm.
    Linear,

    /// Use a repeated RNN search, increasing the radius until enough neighbors
    /// are found.
    ///
    /// This is a stable algorithm.
    ///
    /// Search starts with a radius equal to the radius of the tree divided by
    /// the cardinality of the dataset. If no neighbors are found, the radius is
    /// increased by a factor of 2 until at least one neighbor is found. Then,
    /// the radius is increased by a factor determined by the local fractal
    /// dimension of the neighbors found until enough neighbors are found. This
    /// factor is capped at 2. Once enough neighbors are found, the neighbors
    /// are sorted by distance and the first `k` neighbors are returned. Ties
    /// are broken arbitrarily.
    RepeatedRnn,

    /// Use the knn-sieve, with no separate grains for centers, to perform search.
    ///
    /// This algorithm is not stable.
    ///
    /// For each iteration of the search, we calculate a threshold from the
    /// `Cluster`s such that the k nearest neighbors of the query are guaranteed
    /// to be within the threshold. We then use this threshold to filter out
    /// clusters that are too far away from the query.
    ///
    /// This approach does not treat the center of a cluster separately from the rest
    /// of the points in the cluster.
    SieveV1,

    /// Use the knn-sieve, with separate grains for centers, to perform search.
    ///
    /// This algorithm is not stable.
    ///
    /// For each iteration of the search, we calculate a threshold from the
    /// `Cluster`s such that the k nearest neighbors of the query are guaranteed
    /// to be within the threshold. We then use this threshold to filter out
    /// clusters that are too far away from the query.
    ///
    /// This approach treats the center of a cluster separately from the rest
    /// of the points in the cluster.
    SieveV2,

    /// Uses two priority queues and an increasing threshold to perform search.
    ///
    /// This algorithm is not stable.
    ///
    /// Begins with first priority queue, called `candidates`, wherein the top priority
    /// candidate is the one with the lowest d_min. We use d_min to express the
    /// theoretical closest a point in a given cluster can be to the query. Replaces
    /// the top priority candidate with its children until the top priority candidate
    /// is a leaf. Then, adds all instances in the leaf to a second priority queue, `hits`,
    /// wherein the top priority hit is the one with the highest distance to the query.
    /// Hits are then removed from the queue until the queue has size k. Repeats these steps
    /// until candidates is empty or the closest candidate is worse than the furthest hit.
    ExpandingThreshold,
}

impl Default for Algorithm {
    fn default() -> Self {
        Self::RepeatedRnn
    }
}

impl Algorithm {
    /// Searches for the nearest neighbors of a query.
    ///
    /// # Arguments
    ///
    /// * `query` - The query to search around.
    /// * `k` - The number of neighbors to search for.
    /// * `tree` - The tree to search.
    ///
    /// # Returns
    ///
    /// A vector of 2-tuples, where the first element is the index of the instance
    /// and the second element is the distance from the query to the instance.
    pub(crate) fn search<T, U, D>(self, tree: &Tree<T, U, D>, query: T, k: usize) -> Vec<(usize, U)>
    where
        T: Send + Sync + Copy,
        U: Number,
        D: Dataset<T, U>,
    {
        match self {
            Self::Linear => linear::search(tree.data(), query, k, tree.indices()),
            Self::RepeatedRnn => repeated_rnn::search(tree, query, k),
            Self::SieveV1 => sieve_v1::search(tree, query, k),
            Self::SieveV2 => sieve_v2::search(tree, query, k),
            Self::ExpandingThreshold => expanding_threshold::search(tree, query, k),
        }
    }
}
