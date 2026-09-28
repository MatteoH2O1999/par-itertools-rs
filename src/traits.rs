#[doc(hidden)]
/// Trait to allow for slice specialization.
pub trait ImplType {}

/// Marker type used for standard iterator implementations.
#[non_exhaustive]
pub struct ImplIterator;

impl ImplType for ImplIterator {}

/// Marker type used for slice optimized implementations.
#[non_exhaustive]
pub struct ImplSlice;

impl ImplType for ImplSlice {}

/// An [`Iterator`] and `slice` blanket implementation that provides extra adaptors and
/// methods that work in parallel contexts.
pub trait ParItertools<T, B: ImplType> {
    /// Returns an [`IndexedParallelIterator`](rayon::iter::IndexedParallelIterator) that iterates over
    /// the combinations of the elements from an iterator.
    ///
    /// The iterator produces a new array per iteration, and clones the iterator elements.
    ///
    /// ```
    /// use par_itertools::ParItertools;
    /// use rayon::iter::ParallelIterator;
    ///
    /// let mut combinations = (1..4).combinations::<2>().collect::<Vec<_>>();
    /// let mut expected_combinations = vec![[1, 2], [1, 3], [2, 3]];
    /// combinations.sort();
    /// expected_combinations.sort();
    ///
    /// assert_eq!(combinations, expected_combinations);
    /// ```
    #[cfg(feature = "rayon")]
    fn combinations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;

    /// Returns an [`IndexedParallelIterator`](rayon::iter::IndexedParallelIterator) that iterates over
    /// the combinations of the elements from an iterator, with replacement.
    ///
    /// The iterator produces a new array per iteration, and clones the iterator elements.
    ///
    /// ```
    /// use par_itertools::ParItertools;
    /// use rayon::iter::ParallelIterator;
    ///
    /// let mut combinations = (1..4).combinations_with_replacement::<2>().collect::<Vec<_>>();
    /// let mut expected_combinations = vec![[1, 1], [1, 2], [1, 3], [2, 2], [2, 3], [3, 3]];
    /// combinations.sort();
    /// expected_combinations.sort();
    ///
    /// assert_eq!(combinations, expected_combinations);
    /// ```
    #[cfg(feature = "rayon")]
    fn combinations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;

    /// Returns an [`IndexedParallelIterator`](rayon::iter::IndexedParallelIterator) that iterates over
    /// the permutations of the elements from an iterator.
    ///
    /// The iterator produces a new array per iteration, and clones the iterator elements.
    ///
    /// ```
    /// use par_itertools::ParItertools;
    /// use rayon::iter::ParallelIterator;
    ///
    /// let mut permutations = (1..4).permutations::<2>().collect::<Vec<_>>();
    /// let mut expected_permutations = vec![[1, 2], [1, 3], [2, 1], [2, 3], [3, 1], [3, 2]];
    /// permutations.sort();
    /// expected_permutations.sort();
    ///
    /// assert_eq!(permutations, expected_permutations);
    /// ```
    #[cfg(feature = "rayon")]
    fn permutations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;

    /// Returns an [`IndexedParallelIterator`](rayon::iter::IndexedParallelIterator) that iterates over
    /// the permutations of the elements from an iterator, with replacement.
    ///
    /// The iterator produces a new array per iteration, and clones the iterator elements.
    ///
    /// ```
    /// use par_itertools::ParItertools;
    /// use rayon::iter::ParallelIterator;
    ///
    /// let mut permutations = (1..4).permutations_with_replacement::<2>().collect::<Vec<_>>();
    /// let mut expected_permutations = vec![[1, 1], [1, 2], [1, 3], [2, 1], [2, 2], [2, 3], [3, 1], [3, 2], [3, 3]];
    /// permutations.sort();
    /// expected_permutations.sort();
    ///
    /// assert_eq!(permutations, expected_permutations);
    /// ```
    #[cfg(feature = "rayon")]
    fn permutations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;
}
