use crate::ParItertools;

impl<'a, T: Copy + Sync + Send> ParItertools<T> for &'a [T] {
    #[cfg(feature = "rayon")]
    fn permutations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelPermutationArray::new::<LEN, false>(self)
    }

    #[cfg(feature = "rayon")]
    fn permutations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelPermutationArray::new::<LEN, true>(self)
    }

    #[cfg(feature = "rayon")]
    fn combinations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelCombinationArray::new::<LEN, false>(self)
    }

    #[cfg(feature = "rayon")]
    fn combinations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelCombinationArray::new::<LEN, true>(self)
    }
}
