use crate::{ImplIterator, ImplSlice, ParItertools};

impl<T: Copy + Sync + Send, U: AsRef<[T]> + Send> ParItertools<T, ImplSlice> for U {
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

impl<T: Copy + Sync + Send, U: Iterator<Item = T>> ParItertools<T, ImplIterator> for U {
    #[cfg(feature = "rayon")]
    fn permutations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelPermutationArray::new::<LEN, false>(self.collect::<Box<_>>())
    }

    #[cfg(feature = "rayon")]
    fn permutations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelPermutationArray::new::<LEN, true>(self.collect::<Box<_>>())
    }

    #[cfg(feature = "rayon")]
    fn combinations<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelCombinationArray::new::<LEN, false>(self.collect::<Box<_>>())
    }

    #[cfg(feature = "rayon")]
    fn combinations_with_replacement<const LEN: usize>(
        self,
    ) -> impl rayon::iter::IndexedParallelIterator<Item = [T; LEN]> {
        crate::rayon::ParallelCombinationArray::new::<LEN, true>(self.collect::<Box<_>>())
    }
}
