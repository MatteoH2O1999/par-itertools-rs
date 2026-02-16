use rayon::{
    iter::plumbing::{bridge, Producer},
    prelude::*,
};

fn comb_len<const WITH_REPLACEMENT: bool>(num_elements: usize, mut comb_len: usize) -> usize {
    if comb_len == 0 {
        return 1;
    }

    let mut num_elements = if WITH_REPLACEMENT {
        num_elements + comb_len - 1
    } else {
        num_elements
    };
    let mut size = num_elements;
    let mut i = 1;
    let mut num_combinations = i;

    num_elements -= 1;
    comb_len -= 1;
    i += 1;

    while comb_len > 0 {
        size *= num_elements;
        num_combinations *= i;
        comb_len -= 1;
        i += 1;
        num_elements -= 1;
    }

    debug_assert_eq!(size % num_combinations, 0);

    size / num_combinations
}

pub struct ParallelCombinationArray<
    T,
    A: AsRef<[T]>,
    const LEN: usize,
    const WITH_REPLACEMENT: bool,
> {
    elements: A,
    len: usize,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Copy, A: AsRef<[T]>> ParallelCombinationArray<T, A, 0, false> {
    pub fn new<const LEN: usize, const WITH_REPLACEMENT: bool>(
        elements: A,
    ) -> ParallelCombinationArray<T, A, LEN, WITH_REPLACEMENT> {
        assert!(LEN > 0);
        assert!(WITH_REPLACEMENT || LEN <= elements.as_ref().len());
        ParallelCombinationArray {
            len: comb_len::<WITH_REPLACEMENT>(elements.as_ref().len(), LEN),
            elements,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<
        T: Copy + Send + Sync,
        A: AsRef<[T]> + Send,
        const LEN: usize,
        const WITH_REPLACEMENT: bool,
    > ParallelIterator for ParallelCombinationArray<T, A, LEN, WITH_REPLACEMENT>
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
        T: Copy + Sync + Send,
        A: AsRef<[T]> + Send,
        const LEN: usize,
        const WITH_REPLACEMENT: bool,
    > IndexedParallelIterator for ParallelCombinationArray<T, A, LEN, WITH_REPLACEMENT>
{
    fn len(&self) -> usize {
        self.len
    }

    fn drive<C: rayon::iter::plumbing::Consumer<Self::Item>>(self, consumer: C) -> C::Result {
        bridge(self, consumer)
    }

    fn with_producer<CB: rayon::iter::plumbing::ProducerCallback<Self::Item>>(
        self,
        callback: CB,
    ) -> CB::Output {
        callback.callback(ParallelCombinationArrayProducer::<
            '_,
            T,
            LEN,
            WITH_REPLACEMENT,
        >::new(self.elements.as_ref()))
    }
}

struct ParallelCombinationArrayProducer<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool> {
    elements: &'a [T],
    start_index: usize,
    end_index: usize,
}

impl<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool>
    ParallelCombinationArrayProducer<'a, T, LEN, WITH_REPLACEMENT>
{
    fn new(elements: &'a [T]) -> Self {
        Self {
            elements,
            start_index: 0,
            end_index: comb_len::<WITH_REPLACEMENT>(elements.len(), LEN),
        }
    }
}

impl<'a, T: Sync + Copy, const LEN: usize, const WITH_REPLACEMENT: bool> Producer
    for ParallelCombinationArrayProducer<'a, T, LEN, WITH_REPLACEMENT>
{
    type IntoIter = ParallelCombinationArrayIter<'a, T, LEN, WITH_REPLACEMENT>;
    type Item = [T; LEN];

    fn into_iter(self) -> Self::IntoIter {
        ParallelCombinationArrayIter::new(self.elements, self.start_index, self.end_index)
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

struct ParallelCombinationArrayIter<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool> {
    elements: &'a [T],
    state: ParallelCombinationArrayState<LEN, WITH_REPLACEMENT>,
    end_state: ParallelCombinationArrayState<LEN, WITH_REPLACEMENT>,
    len: usize,
}

impl<'a, T, const LEN: usize, const WITH_REPLACEMENT: bool>
    ParallelCombinationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    fn new(elements: &'a [T], start_index: usize, end_index: usize) -> Self {
        Self {
            elements,
            state: ParallelCombinationArrayState::with_index(elements.len(), start_index),
            end_state: ParallelCombinationArrayState::with_index(elements.len(), end_index),
            len: end_index - start_index,
        }
    }
}

impl<'a, T: Copy, const LEN: usize, const WITH_REPLACEMENT: bool> Iterator
    for ParallelCombinationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    type Item = [T; LEN];

    fn next(&mut self) -> Option<Self::Item> {
        if self.state.index < self.end_state.index {
            let new_perm = self.state.get_comb(self.elements);
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
    for ParallelCombinationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T: Copy, const LEN: usize, const WITH_REPLACEMENT: bool> DoubleEndedIterator
    for ParallelCombinationArrayIter<'a, T, LEN, WITH_REPLACEMENT>
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.end_state.go_back();
        if self.state.index <= self.end_state.index {
            let new_perm = self.end_state.get_comb(self.elements);
            Some(new_perm)
        } else {
            None
        }
    }
}

#[derive(Debug)]
struct ParallelCombinationArrayState<const LEN: usize, const WITH_REPLACEMENT: bool> {
    state: [usize; LEN],
    index: usize,
    depth: usize,
    num_elements: usize,
}

impl<const LEN: usize, const WITH_REPLACEMENT: bool>
    ParallelCombinationArrayState<LEN, WITH_REPLACEMENT>
{
    fn with_index(num_elements: usize, index: usize) -> Self {
        debug_assert!(index <= comb_len::<WITH_REPLACEMENT>(num_elements, LEN));

        let mut state = [0; LEN];

        if index < comb_len::<WITH_REPLACEMENT>(num_elements, LEN) {
            let mut comb_index = index;
            let mut current_index = 0;

            for (i, s) in state.iter_mut().enumerate() {
                loop {
                    let num_combinations = comb_len::<WITH_REPLACEMENT>(
                        num_elements - current_index - if WITH_REPLACEMENT { 0 } else { 1 },
                        LEN - i - 1,
                    );
                    if num_combinations > comb_index {
                        break;
                    }
                    current_index += 1;
                    comb_index -= num_combinations;
                }
                *s = current_index;
                current_index += if WITH_REPLACEMENT { 0 } else { 1 };
            }
        }

        Self {
            state,
            index,
            depth: LEN,
            num_elements,
        }
    }

    fn get_comb<T: Copy>(&self, elements: &[T]) -> [T; LEN] {
        self.state.map(|i| elements[i])
    }

    fn advance(&mut self) {
        debug_assert!(self.index <= comb_len::<WITH_REPLACEMENT>(self.num_elements, LEN));

        self.depth -= 1;
        let mut remainder = true;

        while self.depth < LEN {
            let (previous_state, last) = if self.depth == 0 {
                (0, true)
            } else {
                (self.state[self.depth - 1], false)
            };
            let current_state = &mut self.state[self.depth];

            if remainder {
                *current_state += 1;

                let invalid = if WITH_REPLACEMENT {
                    *current_state >= self.num_elements
                } else {
                    *current_state >= self.num_elements - (LEN - self.depth - 1)
                } && !last;

                if invalid {
                    self.depth -= 1;
                } else {
                    self.depth += 1;
                    remainder = false;
                }
            } else {
                *current_state = previous_state + if WITH_REPLACEMENT { 0 } else { 1 };
                self.depth += 1;
            }
        }

        self.index += 1;
    }

    fn go_back(&mut self) {
        debug_assert!(self.index > 0);

        self.depth -= 1;
        let mut remainder = true;

        while self.depth < LEN {
            let previous_state = if self.depth == 0 {
                0
            } else {
                // Add 1 to avoid using signed type and to make comparison work
                self.state[self.depth - 1] + 1
            };
            let current_state = &mut self.state[self.depth];

            if remainder {
                if *current_state == 0 {
                    *current_state = self.num_elements - 1;
                }

                *current_state -= 1;

                // Add 1 to comparison as described above
                let invalid = if WITH_REPLACEMENT {
                    *current_state + 1 < previous_state
                } else {
                    *current_state < previous_state
                };

                if invalid {
                    self.depth -= 1;
                } else {
                    remainder = false;
                    self.depth += 1;
                }
            } else {
                if WITH_REPLACEMENT {
                    *current_state = self.num_elements - 1;
                } else {
                    *current_state = self.num_elements - (LEN - self.depth);
                }
                self.depth += 1;
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
        fn combination_lengths() {
            assert_eq!(comb_len::<false>(5, 3), 10);
            assert_eq!(comb_len::<false>(10, 4), 210);
        }

        #[test]
        fn state_creation_with_index() {
            let s = ParallelCombinationArrayState::<3, false>::with_index(5, 1);
            assert_eq!(s.state, [0, 1, 3]);

            let s = ParallelCombinationArrayState::<3, false>::with_index(5, 3);
            assert_eq!(s.state, [0, 2, 3]);

            let s = ParallelCombinationArrayState::<3, false>::with_index(5, 5);
            assert_eq!(s.state, [0, 3, 4]);

            let s = ParallelCombinationArrayState::<3, false>::with_index(5, 9);
            assert_eq!(s.state, [2, 3, 4]);
        }

        #[test]
        fn state_advance() {
            let mut s = ParallelCombinationArrayState::<3, false>::with_index(5, 0);
            assert_eq!(s.state, [0, 1, 2]);
            assert_eq!(s.index, 0);
            s.advance();
            assert_eq!(s.state, [0, 1, 3]);
            assert_eq!(s.index, 1);
            s.advance();
            assert_eq!(s.state, [0, 1, 4]);
            assert_eq!(s.index, 2);
            s.advance();
            assert_eq!(s.state, [0, 2, 3]);
            assert_eq!(s.index, 3);
            s.advance();
            assert_eq!(s.state, [0, 2, 4]);
            assert_eq!(s.index, 4);
            s.advance();
            assert_eq!(s.state, [0, 3, 4]);
            assert_eq!(s.index, 5);
            s.advance();
            assert_eq!(s.state, [1, 2, 3]);
            assert_eq!(s.index, 6);
            s.advance();
            assert_eq!(s.state, [1, 2, 4]);
            assert_eq!(s.index, 7);
            s.advance();
            assert_eq!(s.state, [1, 3, 4]);
            assert_eq!(s.index, 8);
            s.advance();
            assert_eq!(s.state, [2, 3, 4]);
            assert_eq!(s.index, 9);
        }

        #[test]
        fn state_go_back() {
            let mut s = ParallelCombinationArrayState::<3, false>::with_index(5, 9);
            assert_eq!(s.state, [2, 3, 4]);
            assert_eq!(s.index, 9);
            s.go_back();
            assert_eq!(s.state, [1, 3, 4]);
            assert_eq!(s.index, 8);
            s.go_back();
            assert_eq!(s.state, [1, 2, 4]);
            assert_eq!(s.index, 7);
            s.go_back();
            assert_eq!(s.state, [1, 2, 3]);
            assert_eq!(s.index, 6);
            s.go_back();
            assert_eq!(s.state, [0, 3, 4]);
            assert_eq!(s.index, 5);
            s.go_back();
            assert_eq!(s.state, [0, 2, 4]);
            assert_eq!(s.index, 4);
            s.go_back();
            assert_eq!(s.state, [0, 2, 3]);
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
        fn simple_combination() {
            let simple_combination = ParallelCombinationArray::new::<3, false>([1, 2, 3, 4]);
            let mut actual = simple_combination.collect::<Vec<_>>();
            actual.sort();
            assert_eq!(actual, vec![[1, 2, 3], [1, 2, 4], [1, 3, 4], [2, 3, 4]]);
        }

        #[test]
        fn long_combination() {
            let long_combination =
                ParallelCombinationArray::new::<3, false>((START..END).collect::<Box<_>>());
            let mut actual = long_combination.collect::<Vec<_>>();
            actual.sort();

            let mut expected = (START..END)
                .flat_map(|i| {
                    ((i + 1)..END).flat_map(move |j| ((j + 1)..END).map(move |k| [i, j, k]))
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
        fn combination_lengths() {
            assert_eq!(comb_len::<true>(5, 3), 35);
            assert_eq!(comb_len::<true>(10, 3), 220);
        }

        #[test]
        fn state_creation_with_index() {
            let s = ParallelCombinationArrayState::<3, true>::with_index(5, 1);
            assert_eq!(s.state, [0, 0, 1]);

            let s = ParallelCombinationArrayState::<3, true>::with_index(5, 3);
            assert_eq!(s.state, [0, 0, 3]);

            let s = ParallelCombinationArrayState::<3, true>::with_index(5, 30);
            assert_eq!(s.state, [2, 4, 4]);

            let s = ParallelCombinationArrayState::<3, true>::with_index(5, 31);
            assert_eq!(s.state, [3, 3, 3]);
        }

        #[test]
        fn state_advance() {
            let mut s = ParallelCombinationArrayState::<3, true>::with_index(5, 0);
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
            assert_eq!(s.state, [0, 1, 1]);
            assert_eq!(s.index, 5);
            s.advance();
            assert_eq!(s.state, [0, 1, 2]);
            assert_eq!(s.index, 6);
            s.advance();
            assert_eq!(s.state, [0, 1, 3]);
            assert_eq!(s.index, 7);
            s.advance();
            assert_eq!(s.state, [0, 1, 4]);
            assert_eq!(s.index, 8);
            s.advance();
            assert_eq!(s.state, [0, 2, 2]);
            assert_eq!(s.index, 9);
            s.advance();
            assert_eq!(s.state, [0, 2, 3]);
            assert_eq!(s.index, 10);
            s.advance();
            assert_eq!(s.state, [0, 2, 4]);
            assert_eq!(s.index, 11);
            s.advance();
            assert_eq!(s.state, [0, 3, 3]);
            assert_eq!(s.index, 12);
            s.advance();
            assert_eq!(s.state, [0, 3, 4]);
            assert_eq!(s.index, 13);
        }

        #[test]
        fn state_go_back() {
            let mut s = ParallelCombinationArrayState::<3, true>::with_index(5, 34);
            assert_eq!(s.state, [4, 4, 4]);
            assert_eq!(s.index, 34);
            s.go_back();
            assert_eq!(s.state, [3, 4, 4]);
            assert_eq!(s.index, 33);
            s.go_back();
            assert_eq!(s.state, [3, 3, 4]);
            assert_eq!(s.index, 32);
            s.go_back();
            assert_eq!(s.state, [3, 3, 3]);
            assert_eq!(s.index, 31);
            s.go_back();
            assert_eq!(s.state, [2, 4, 4]);
            assert_eq!(s.index, 30);
            s.go_back();
            assert_eq!(s.state, [2, 3, 4]);
            assert_eq!(s.index, 29);
            s.go_back();
            assert_eq!(s.state, [2, 3, 3]);
            assert_eq!(s.index, 28);
            s.go_back();
            assert_eq!(s.state, [2, 2, 4]);
            assert_eq!(s.index, 27);
            s.go_back();
            assert_eq!(s.state, [2, 2, 3]);
            assert_eq!(s.index, 26);
            s.go_back();
            assert_eq!(s.state, [2, 2, 2]);
            assert_eq!(s.index, 25);
            s.go_back();
            assert_eq!(s.state, [1, 4, 4]);
            assert_eq!(s.index, 24);
            s.go_back();
            assert_eq!(s.state, [1, 3, 4]);
            assert_eq!(s.index, 23);
            s.go_back();
            assert_eq!(s.state, [1, 3, 3]);
            assert_eq!(s.index, 22);
            s.go_back();
            assert_eq!(s.state, [1, 2, 4]);
            assert_eq!(s.index, 21);
        }

        #[test]
        fn simple_combination() {
            let simple_combination = ParallelCombinationArray::new::<3, true>([1, 2, 3]);
            let mut actual = simple_combination.collect::<Vec<_>>();
            actual.sort();
            assert_eq!(
                actual,
                vec![
                    [1, 1, 1],
                    [1, 1, 2],
                    [1, 1, 3],
                    [1, 2, 2],
                    [1, 2, 3],
                    [1, 3, 3],
                    [2, 2, 2],
                    [2, 2, 3],
                    [2, 3, 3],
                    [3, 3, 3]
                ]
            );
        }

        #[test]
        fn long_combination() {
            let long_combination =
                ParallelCombinationArray::new::<3, true>((START..END).collect::<Box<_>>());
            let mut actual = long_combination.collect::<Vec<_>>();
            actual.sort();

            let mut expected = (START..END)
                .flat_map(|i| (i..END).flat_map(move |j| (j..END).map(move |k| [i, j, k])))
                .collect::<Vec<_>>();
            expected.sort();

            assert_eq!(actual, expected);
        }
    }
}
