// Disclosure: AI wrote this
pub fn merge_in_place<T>(a: &mut Vec<T>, b: &[T])
where
    T: Copy + Clone + Default + PartialOrd,
{
    let mut i = a.len();
    let mut j = b.len();
    a.resize(i + j, T::default());

    // write index
    let mut k = a.len();

    // merge backwards
    while i > 0 && j > 0 {
        k -= 1;
        if a[i - 1] > b[j - 1] {
            i -= 1;
            a[k] = a[i];
        } else {
            j -= 1;
            a[k] = b[j];
        }
    }

    // Copy any remaining from b
    while j > 0 {
        j -= 1;
        k -= 1;
        a[k] = b[j];
    }
}
