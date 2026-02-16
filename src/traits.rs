pub trait ParItertools<T> {
    #[cfg(feature = "rayon")]
    fn combinations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;

    #[cfg(feature = "rayon")]
    fn combinations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;

    #[cfg(feature = "rayon")]
    fn permutations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;

    #[cfg(feature = "rayon")]
    fn permutations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]>;
}
