use rayon::{
    iter::plumbing::{bridge, Producer},
    prelude::*,
};

const fn perm_len<const WITH_REPLACEMENT: bool>(
    mut num_elements: usize,
    mut perm_len: usize,
) -> usize {
    if perm_len == 0 {
        return 1;
    }

    let mut size = num_elements;

    if !WITH_REPLACEMENT {
        num_elements -= 1;
    }
    perm_len -= 1;

    while perm_len > 0 {
        size *= num_elements;
        perm_len -= 1;
        if !WITH_REPLACEMENT {
            num_elements -= 1;
        }
    }

    size
}

pub struct ParallelPermutationArray<
    T,
    A: AsRef<[T]>,
    const LEN: usize,
    const WITH_REPLACEMENT: bool,
> {
    elements: A,
    len: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Copy, A: AsRef<[T]>> ParallelPermutationArray<T, A, 0, false> {
    pub fn new<const LEN: usize, const WITH_REPLACEMENT: bool>(
        elements: A,
    ) -> ParallelPermutationArray<T, A, LEN, WITH_REPLACEMENT> {
        assert!(LEN > 0);
        assert!(WITH_REPLACEMENT || LEN <= elements.as_ref().len());
        ParallelPermutationArray {
            len: perm_len::<WITH_REPLACEMENT>(elements.as_ref().len(), LEN),
            elements,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<
        T: Send + Sync + Copy,
        A: AsRef<[T]> + Send,
        const LEN: usize,
        const WITH_REPLACEMENT: bool,
    > ParallelIterator for ParallelPermutationArray<T, A, LEN, WITH_REPLACEMENT>
{
    type Item = [T; LEN];

    fn drive_unindexed<C>(self, consumer: C) -> C::Result
    where
        C: rayon::iter::plumbing::UnindexedConsumer<Self::Item>,
    {
        bridge(self, consumer)
    }

    fn opt_len(&self) -> Option<usize> {
        Some(self.len)
    }
}

impl<
        T: Send + Sync + Copy,
        A: AsRef<[T]> + Send,
        const LEN: usize,
        const WITH_REPLACEMENT: bool,
    > IndexedParallelIterator for ParallelPermutationArray<T, A, LEN, WITH_REPLACEMENT>
{
    fn len(&self) -> usize {
        self.len
    }

    fn with_producer<CB: rayon::iter::plumbing::ProducerCallback<Self::Item>>(
        self,
        callback: CB,
    ) -> CB::Output {
        callback.callback(ParallelPermutationArrayProducer::<
            '_,
            T,
            LEN,
            WITH_REPLACEMENT,
        >::new(self.elements.as_ref()))
    }

    fn drive<C: rayon::iter::plumbing::Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }
}

struct ParallelPermutationArrayProducer<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool> {
    elements: &'a [T],
    start_index: usize,
    end_index: usize,
}

impl<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool>
    ParallelPermutationArrayProducer<'a, T, LEN, WITH_REPLACEMENT>
{
    const fn new(elements: &'a [T]) -> Self {
        Self {
            elements,
            start_index: 0,
            end_index: perm_len::<WITH_REPLACEMENT>(elements.len(), LEN),
        }
    }
}

impl<'a, T: Copy + Sync, const LEN: usize, const WITH_REPLACEMENT: bool> Producer
    for ParallelPermutationArrayProducer<'a, T, LEN, WITH_REPLACEMENT>
{
    type IntoIter = ParallelPermutationArrayIter<'a, T, LEN, WITH_REPLACEMENT>;
    type Item = [T; LEN];

    fn into_iter(self) -> Self::IntoIter {
        ParallelPermutationArrayIter::new(self.elements, self.start_index, self.end_index)
    }

    fn split_at(mut self, index: usize) -> (Self, Self) {
        let index = self.start_index + index;
        let other = Self {
            elements: self.elements,
            end_index: self.end_index,
            start_index: index,
        };
        self.end_index = index;
        (self, other)
    }
}

struct ParallelPermutationArrayIter<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool> {
    elements: &'a [T],
    state: ParallelPermutationArrayState<LEN, WITH_REPLACEMENT>,
    end_state: ParallelPermutationArrayState<LEN, WITH_REPLACEMENT>,
    len: usize,
}

impl<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool>
    ParallelPermutationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    fn new(elements: &'a [T], start_index: usize, end_index: usize) -> Self {
        Self {
            elements,
            state: ParallelPermutationArrayState::with_index(elements.len(), start_index),
            end_state: ParallelPermutationArrayState::with_index(elements.len(), end_index),
            len: end_index - start_index,
        }
    }
}

impl<'a, T: Copy, const LEN: usize, const WITH_REPLACEMENT: bool> Iterator
    for ParallelPermutationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    type Item = [T; LEN];

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.index < self.end_state.index {
            let new_perm = self.state.get_perm(self.elements);
            self.state.advance();
            Some(new_perm)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T: Copy, const LEN: usize, const WITH_REPLACEMENT: bool> ExactSizeIterator
    for ParallelPermutationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T: Copy, const LEN: usize, const WITH_REPLACEMENT: bool> DoubleEndedIterator
    for ParallelPermutationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.end_state.go_back();
        if self.state.index <= self.end_state.index {
            let new_perm = self.end_state.get_perm(self.elements);
            Some(new_perm)
        } else {
            None
        }
    }
}

struct ParallelPermutationArrayState<const LEN: usize, const WITH_REPLACEMENT: bool> {
    state: [usize; LEN],
    used: Box<[usize]>,
    index: usize,
    num_elements: usize,
}

impl<const LEN: usize, const WITH_REPLACEMENT: bool>
    ParallelPermutationArrayState<LEN, WITH_REPLACEMENT>
{
    fn with_index(num_elements: usize, index: usize) -> Self {
        debug_assert!(index <= perm_len::<WITH_REPLACEMENT>(num_elements, LEN));

        let mut state = [0; LEN];
        let mut used = if !WITH_REPLACEMENT {
            vec![0; num_elements].into_boxed_slice()
        } else {
            Box::default()
        };

        if index < perm_len::<WITH_REPLACEMENT>(num_elements, LEN) {
            let mut counter = [0; LEN];
            let mut perm_index = index;

            for (i, c) in counter.iter_mut().enumerate() {
                let n = if WITH_REPLACEMENT {
                    num_elements
                } else {
                    num_elements - i - 1
                };
                let divisor = perm_len::<WITH_REPLACEMENT>(n, LEN - i - 1);
                *c = perm_index / divisor;
                perm_index -= *c * divisor;
            }

            for (s, &c) in state.iter_mut().zip(counter.iter()) {
                *s = (0..num_elements)
                    .filter(|&i| if WITH_REPLACEMENT { true } else { used[i] == 0 })
                    .skip(c)
                    .next()
                    .expect("Should be able to choose an element");
                if !WITH_REPLACEMENT {
                    used[*s] = 1;
                }
            }
        }

        Self {
            state,
            used,
            index,
            num_elements,
        }
    }

    fn get_perm<T: Copy>(&self, elements: &[T]) -> [T; LEN] {
        self.state.map(|i| elements[i])
    }

    fn advance(&mut self) {
        let mut valid = false;

        while !valid {
            let num_elements = self.num_elements;
            let mut remainder = true;
            valid = true;

            for elem in self.state.iter_mut().rev() {
                if remainder {
                    if !WITH_REPLACEMENT {
                        self.used[*elem] -= 1;
                    }
                    *elem += 1;
                    if *elem == num_elements {
                        *elem = 0;
                    } else {
                        remainder = false;
                    }
                    if !WITH_REPLACEMENT {
                        self.used[*elem] += 1;
                    }
                }
            }

            if !WITH_REPLACEMENT {
                for elem in self.state {
                    if self.used[elem] > 1 {
                        valid = false
                    }
                }
            }
        }

        self.index += 1;
    }

    fn go_back(&mut self) {
        let num_elements = self.num_elements;
        let mut valid = false;

        while !valid {
            let mut remainder = true;
            valid = true;

            for elem in self.state.iter_mut().rev() {
                if remainder {
                    if !WITH_REPLACEMENT {
                        self.used[*elem] -= 1;
                    }
                    if *elem == 0 {
                        *elem = num_elements - 1;
                    } else {
                        *elem -= 1;
                        remainder = false;
                    }
                    if !WITH_REPLACEMENT {
                        self.used[*elem] += 1;
                    }
                }
            }

            if !WITH_REPLACEMENT {
                for elem in self.state {
                    if self.used[elem] > 1 {
                        valid = false
                    }
                }
            }
        }

        self.index -= 1;
    }
}

#[cfg(test)]
mod test {
    const START: usize = 0;
    const END: usize = 25;

    mod without_replacement {
        use super::super::*;
        use super::*;

        #[test]
        fn permutation_lengths() {
            assert_eq!(perm_len::<false>(5, 3), 60);
            assert_eq!(perm_len::<false>(10, 3), 720);
        }

        #[test]
        fn state_creation_with_index() {
            let s = ParallelPermutationArrayState::<3, false>::with_index(5, 1);
            assert_eq!(s.state, [0, 1, 3]);

            let s = ParallelPermutationArrayState::<3, false>::with_index(5, 3);
            assert_eq!(s.state, [0, 2, 1]);

            let s = ParallelPermutationArrayState::<3, false>::with_index(5, 42);
            assert_eq!(s.state, [3, 2, 0]);

            let s = ParallelPermutationArrayState::<3, false>::with_index(5, 43);
            assert_eq!(s.state, [3, 2, 1]);
        }

        #[test]
        fn state_advance() {
            let mut s = ParallelPermutationArrayState::<3, false>::with_index(5, 0);
            assert_eq!(s.state, [0, 1, 2]);
            assert_eq!(s.index, 0);
            s.advance();
            assert_eq!(s.state, [0, 1, 3]);
            assert_eq!(s.index, 1);
            s.advance();
            assert_eq!(s.state, [0, 1, 4]);
            assert_eq!(s.index, 2);
            s.advance();
            assert_eq!(s.state, [0, 2, 1]);
            assert_eq!(s.index, 3);
            s.advance();
            assert_eq!(s.state, [0, 2, 3]);
            assert_eq!(s.index, 4);
            s.advance();
            assert_eq!(s.state, [0, 2, 4]);
            assert_eq!(s.index, 5);
            s.advance();
            assert_eq!(s.state, [0, 3, 1]);
            assert_eq!(s.index, 6);
            s.advance();
            assert_eq!(s.state, [0, 3, 2]);
            assert_eq!(s.index, 7);
            s.advance();
            assert_eq!(s.state, [0, 3, 4]);
            assert_eq!(s.index, 8);
            s.advance();
            assert_eq!(s.state, [0, 4, 1]);
            assert_eq!(s.index, 9);
            s.advance();
            assert_eq!(s.state, [0, 4, 2]);
            assert_eq!(s.index, 10);
            s.advance();
            assert_eq!(s.state, [0, 4, 3]);
            assert_eq!(s.index, 11);
            s.advance();
            assert_eq!(s.state, [1, 0, 2]);
            assert_eq!(s.index, 12);
            s.advance();
            assert_eq!(s.state, [1, 0, 3]);
            assert_eq!(s.index, 13);
        }

        #[test]
        fn state_go_back() {
            let mut s = ParallelPermutationArrayState::<3, false>::with_index(5, 12);
            assert_eq!(s.state, [1, 0, 2]);
            assert_eq!(s.index, 12);
            s.go_back();
            assert_eq!(s.state, [0, 4, 3]);
            assert_eq!(s.index, 11);
            s.go_back();
            assert_eq!(s.state, [0, 4, 2]);
            assert_eq!(s.index, 10);
            s.go_back();
            assert_eq!(s.state, [0, 4, 1]);
            assert_eq!(s.index, 9);
            s.go_back();
            assert_eq!(s.state, [0, 3, 4]);
            assert_eq!(s.index, 8);
            s.go_back();
            assert_eq!(s.state, [0, 3, 2]);
            assert_eq!(s.index, 7);
            s.go_back();
            assert_eq!(s.state, [0, 3, 1]);
            assert_eq!(s.index, 6);
            s.go_back();
            assert_eq!(s.state, [0, 2, 4]);
            assert_eq!(s.index, 5);
            s.go_back();
            assert_eq!(s.state, [0, 2, 3]);
            assert_eq!(s.index, 4);
            s.go_back();
            assert_eq!(s.state, [0, 2, 1]);
            assert_eq!(s.index, 3);
            s.go_back();
            assert_eq!(s.state, [0, 1, 4]);
            assert_eq!(s.index, 2);
            s.go_back();
            assert_eq!(s.state, [0, 1, 3]);
            assert_eq!(s.index, 1);
            s.go_back();
            assert_eq!(s.state, [0, 1, 2]);
            assert_eq!(s.index, 0);
        }

        #[test]
        fn simple_permutation() {
            let simple_permutation = ParallelPermutationArray::new::<3, false>([1, 2, 3, 4]);
            let mut actual = simple_permutation.collect::<Vec<_>>();
            actual.sort();
            assert_eq!(
                actual,
                vec![
                    [1, 2, 3],
                    [1, 2, 4],
                    [1, 3, 2],
                    [1, 3, 4],
                    [1, 4, 2],
                    [1, 4, 3],
                    [2, 1, 3],
                    [2, 1, 4],
                    [2, 3, 1],
                    [2, 3, 4],
                    [2, 4, 1],
                    [2, 4, 3],
                    [3, 1, 2],
                    [3, 1, 4],
                    [3, 2, 1],
                    [3, 2, 4],
                    [3, 4, 1],
                    [3, 4, 2],
                    [4, 1, 2],
                    [4, 1, 3],
                    [4, 2, 1],
                    [4, 2, 3],
                    [4, 3, 1],
                    [4, 3, 2]
                ]
            );
        }

        #[test]
        fn long_permutation() {
            let long_permutation =
                ParallelPermutationArray::new::<3, false>((START..END).collect::<Box<_>>());
            let mut actual = long_permutation.collect::<Vec<_>>();
            actual.sort();

            let mut expected = (START..END)
                .flat_map(|i| {
                    (START..END).flat_map(move |j| {
                        (START..END).filter_map(move |k| {
                            if i != j && i != k && j != k {
                                Some([i, j, k])
                            } else {
                                None
                            }
                        })
                    })
                })
                .collect::<Vec<_>>();
            expected.sort();

            assert_eq!(actual, expected);
        }
    }

    mod with_replacement {
        use super::super::*;
        use super::*;

        #[test]
        fn permutation_lengths() {
            assert_eq!(perm_len::<true>(5, 3), 125);
            assert_eq!(perm_len::<true>(10, 3), 1000);
        }

        #[test]
        fn state_creation_with_index() {
            let s = ParallelPermutationArrayState::<3, true>::with_index(5, 1);
            assert_eq!(s.state, [0, 0, 1]);

            let s = ParallelPermutationArrayState::<3, true>::with_index(5, 3);
            assert_eq!(s.state, [0, 0, 3]);

            let s = ParallelPermutationArrayState::<3, true>::with_index(5, 42);
            assert_eq!(s.state, [1, 3, 2]);

            let s = ParallelPermutationArrayState::<3, true>::with_index(5, 43);
            assert_eq!(s.state, [1, 3, 3]);
        }

        #[test]
        fn state_advance() {
            let mut s = ParallelPermutationArrayState::<3, true>::with_index(5, 0);
            assert_eq!(s.state, [0, 0, 0]);
            assert_eq!(s.index, 0);
            s.advance();
            assert_eq!(s.state, [0, 0, 1]);
            assert_eq!(s.index, 1);
            s.advance();
            assert_eq!(s.state, [0, 0, 2]);
            assert_eq!(s.index, 2);
            s.advance();
            assert_eq!(s.state, [0, 0, 3]);
            assert_eq!(s.index, 3);
            s.advance();
            assert_eq!(s.state, [0, 0, 4]);
            assert_eq!(s.index, 4);
            s.advance();
            assert_eq!(s.state, [0, 1, 0]);
            assert_eq!(s.index, 5);
            s.advance();
            assert_eq!(s.state, [0, 1, 1]);
            assert_eq!(s.index, 6);
            s.advance();
            assert_eq!(s.state, [0, 1, 2]);
            assert_eq!(s.index, 7);
            s.advance();
            assert_eq!(s.state, [0, 1, 3]);
            assert_eq!(s.index, 8);
            s.advance();
            assert_eq!(s.state, [0, 1, 4]);
            assert_eq!(s.index, 9);
            s.advance();
            assert_eq!(s.state, [0, 2, 0]);
            assert_eq!(s.index, 10);
            s.advance();
            assert_eq!(s.state, [0, 2, 1]);
            assert_eq!(s.index, 11);
            s.advance();
            assert_eq!(s.state, [0, 2, 2]);
            assert_eq!(s.index, 12);
            s.advance();
            assert_eq!(s.state, [0, 2, 3]);
            assert_eq!(s.index, 13);
        }

        #[test]
        fn state_go_back() {
            let mut s = ParallelPermutationArrayState::<3, true>::with_index(5, 12);
            assert_eq!(s.state, [0, 2, 2]);
            assert_eq!(s.index, 12);
            s.go_back();
            assert_eq!(s.state, [0, 2, 1]);
            assert_eq!(s.index, 11);
            s.go_back();
            assert_eq!(s.state, [0, 2, 0]);
            assert_eq!(s.index, 10);
            s.go_back();
            assert_eq!(s.state, [0, 1, 4]);
            assert_eq!(s.index, 9);
            s.go_back();
            assert_eq!(s.state, [0, 1, 3]);
            assert_eq!(s.index, 8);
            s.go_back();
            assert_eq!(s.state, [0, 1, 2]);
            assert_eq!(s.index, 7);
            s.go_back();
            assert_eq!(s.state, [0, 1, 1]);
            assert_eq!(s.index, 6);
            s.go_back();
            assert_eq!(s.state, [0, 1, 0]);
            assert_eq!(s.index, 5);
            s.go_back();
            assert_eq!(s.state, [0, 0, 4]);
            assert_eq!(s.index, 4);
            s.go_back();
            assert_eq!(s.state, [0, 0, 3]);
            assert_eq!(s.index, 3);
            s.go_back();
            assert_eq!(s.state, [0, 0, 2]);
            assert_eq!(s.index, 2);
            s.go_back();
            assert_eq!(s.state, [0, 0, 1]);
            assert_eq!(s.index, 1);
            s.go_back();
            assert_eq!(s.state, [0, 0, 0]);
            assert_eq!(s.index, 0);
        }

        #[test]
        fn simple_perm() {
            let simple_perm = ParallelPermutationArray::new::<3, true>([0, 1, 2]);
            let mut v = simple_perm.collect::<Vec<_>>();
            v.sort();
            assert_eq!(
                v,
                vec![
                    [0, 0, 0],
                    [0, 0, 1],
                    [0, 0, 2],
                    [0, 1, 0],
                    [0, 1, 1],
                    [0, 1, 2],
                    [0, 2, 0],
                    [0, 2, 1],
                    [0, 2, 2],
                    [1, 0, 0],
                    [1, 0, 1],
                    [1, 0, 2],
                    [1, 1, 0],
                    [1, 1, 1],
                    [1, 1, 2],
                    [1, 2, 0],
                    [1, 2, 1],
                    [1, 2, 2],
                    [2, 0, 0],
                    [2, 0, 1],
                    [2, 0, 2],
                    [2, 1, 0],
                    [2, 1, 1],
                    [2, 1, 2],
                    [2, 2, 0],
                    [2, 2, 1],
                    [2, 2, 2]
                ]
            );
        }

        #[test]
        fn long_permutation() {
            let long_permutation =
                ParallelPermutationArray::new::<3, true>((START..END).collect::<Box<_>>());
            let mut actual = long_permutation.collect::<Vec<_>>();
            actual.sort();

            let mut expected = (START..END)
                .flat_map(|i| (START..END).flat_map(move |j| (START..END).map(move |k| [i, j, k])))
                .collect::<Vec<_>>();
            expected.sort();

            assert_eq!(actual, expected);
        }
    }
}
