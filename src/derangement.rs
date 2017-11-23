use rand::{thread_rng, Rng};

// This algorithm has been found on stack overflow <https://stackoverflow.com/a/25238398>.
// Its correctness has been empirically validated
// The uniformity of the generated derangments' distribution has yet to be
// tested/validated/proven.
pub fn random_derangement(n: usize) -> Vec<usize> {
    let mut rng = thread_rng();
    'outer: loop {
        let mut range: Vec<usize> = (0..n).collect();
        for i in (0..n).rev() {
            let p = rng.gen_range(0, i + 1);
            if range[p] == i {
                continue 'outer;
            }
            range.swap(i, p);
        }
        if range[0] != 0 {
            return range;
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    // Ensures that no fixed point are present in the generated vector.
    fn no_fixed_point() {

        let mut rng = thread_rng();

        for _ in 0..10000 {

            let derangement = random_derangement(rng.gen_range(5,10));

            // Test that all elements are different than their index.
            let not_found = derangement
                .into_iter()
                .enumerate()
                .all(|(i,p)| i != p);

            assert!(not_found);

        }
    }

}
