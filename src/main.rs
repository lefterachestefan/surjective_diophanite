use binary_search::{Direction, binary_search};
use rayon::prelude::*;
use std::{mem::MaybeUninit, time::Instant};

const CHECK_UP_TO: usize = 3000;
type SurjectiveLine = [u64; CHECK_UP_TO + 1];

// better = bigger precision, but might not fit in u64
// const BIG_PRIME: u64 = 12345678910987654321u64;
const BIG_PRIME: u64 = 100000000003;

#[inline(always)]
fn next_m(prev: &mut SurjectiveLine, m: usize) {
    const MIN_WINDOW: usize = 32;
    const THREADS: usize = 512;
    let chunk_size = (m / THREADS).max(MIN_WINDOW);
    let mut last_clones: [MaybeUninit<u64>; THREADS + MIN_WINDOW] =
        unsafe { MaybeUninit::uninit().assume_init() };

    unsafe { last_clones.get_unchecked_mut(0..(m / chunk_size + 1)) }
        .par_iter_mut()
        .enumerate()
        .for_each(|(i, v)| unsafe {
            v.write(*prev.get_unchecked(i * chunk_size));
        });
    // let last_clones = unsafe { MaybeUninit::array_assume_init(last_clones) };
    let last_clones = unsafe {
        std::mem::transmute::<[MaybeUninit<u64>; THREADS + MIN_WINDOW], [u64; THREADS + MIN_WINDOW]>(
            last_clones,
        )
    };

    unsafe { prev.get_unchecked_mut(1..=m) }
        .chunks_mut(chunk_size)
        .enumerate()
        .for_each(|(i, slice)| {
            let mut last: u64 = unsafe { *last_clones.get_unchecked(i) };
            let n = i * chunk_size + 1;
            for (i, v) in slice.iter_mut().enumerate() {
                last = std::mem::replace(v, (((n + i) as u64) * (last + *v)) % BIG_PRIME)
            }
        });
}

#[inline(always)]
const fn get_start_line() -> SurjectiveLine {
    let mut surjective: [u64; CHECK_UP_TO + 1] = const { [0u64; CHECK_UP_TO + 1] };
    surjective[1] = 1;
    surjective
}

fn main() {
    let start_time = Instant::now();
    let mut last_check_time = start_time;
    let mut surjective = const { get_start_line() };
    for m in 2..=CHECK_UP_TO {
        next_m(&mut surjective, m);
        let change_point = binary_search((1, ()), (m - 1, ()), |i| unsafe {
            if surjective.get_unchecked(i) <= surjective.get_unchecked(i + 1) {
                Direction::Low(())
            } else {
                Direction::High(())
            }
        })
        .0
        .0 + 1;
        let solution = (2..=m).into_par_iter().find_map_any(|z| unsafe {
            let x = z - 1;
            let right_side = surjective.get_unchecked(z);
            let x_side = surjective.get_unchecked(x);
            let (left_result, right_result) = rayon::join(
                || {
                    surjective[1..=change_point]
                        .binary_search_by(|i| (x_side + i).cmp(right_side))
                        .is_ok()
                },
                || {
                    surjective[change_point + 1..=m]
                        .binary_search_by(|i| (x_side + i).cmp(right_side).reverse())
                        .is_ok()
                },
            );
            if left_result || right_result {
                return Some(());
            }
            None
        });
        if solution.is_some() {
            println!("FOUND FOR {m}");
        }
        if last_check_time.elapsed().as_secs_f64() >= 10.0 {
            last_check_time = Instant::now();
            println!("{:.2}%", (m * 100) as f64 / CHECK_UP_TO as f64)
        }
    }
    println!(
        "Program checked up to {CHECK_UP_TO} in {:.3} seconds.",
        start_time.elapsed().as_secs_f64()
    );
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn gen_lines() {
        let mut line = get_start_line();
        for i in 2..=CHECK_UP_TO {
            next_m(&mut line, i);
        }
        println!("{}", size_of_val(&line));
    }
}
