use std::collections::HashSet;
use std::hash::BuildHasher;
use std::hash::Hash;

pub(crate) fn extend<T, S, I: IntoIterator<Item = T>>(
    set: &mut HashSet<T, S>,
    iter: I,
) -> bool
where
    T: Eq + Hash,
    S: BuildHasher,
{
    let before_len = set.len();
    set.extend(iter);
    let after_len = set.len();

    before_len != after_len
}
