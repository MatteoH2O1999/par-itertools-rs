#![cfg(feature = "rayon")]

use par_itertools::ParItertools;
use rayon::iter::ParallelIterator;

#[test]
fn rayon_combinations_without_repetitions_on_array() {
    let array = [0, 1, 2];
    let mut expected_combinations = [[0, 1], [1, 2], [0, 2]];
    expected_combinations.sort();

    let mut actual_combinations = array.combinations::<2>().collect::<Box<_>>();
    actual_combinations.sort();

    assert_eq!(expected_combinations.as_ref(), actual_combinations.as_ref());
}
