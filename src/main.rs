use binary_search::{Direction, binary_search};
use rayon::prelude::*;
use rug::{Complete, Integer};
use std::time::Instant;

const CHECK_UP_TO: usize = 3000;

fn next_m(prev: &mut [Integer; CHECK_UP_TO + 1], m: usize) {
    let mut last: Integer = 0.into();
    for n in 1..=m {
        unsafe {
            let new_last = prev.get_unchecked(n).clone();
            *prev.get_unchecked_mut(n) = n * (prev.get_unchecked(n).clone() + last);
            last = new_last;
        }
    }
    // dbg!(&prev);
}

fn main() {
    let start_time = Instant::now();
    let mut last_check_time = start_time;
    let mut surjective: [Integer; CHECK_UP_TO + 1] = std::array::from_fn(|_| Integer::from(0));
    surjective[1] = Integer::from(1);
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
            let st = binary_search((1, ()), (change_point, ()), |i| {
                if &(x_side + surjective.get_unchecked(i)).complete() <= right_side {
                    Direction::Low(())
                } else {
                    Direction::High(())
                }
            })
            .0
            .0;
            if &(x_side + surjective.get_unchecked(st)).complete() == right_side {
                return Some((x, st, z));
            }
            let st = binary_search((change_point + 1, ()), (m, ()), |i| {
                if &(x_side + surjective.get_unchecked(i)).complete() >= right_side {
                    Direction::Low(())
                } else {
                    Direction::High(())
                }
            })
            .0
            .0;
            if &(x_side + surjective.get_unchecked(st)).complete() == right_side {
                return Some((x, st, z));
            }
            None
        });
        if let Some(sol) = solution {
            println!("FOUND FOR {m}: {sol:?}");
        }
        if last_check_time.elapsed().as_secs_f64() >= 10.0 {
            last_check_time = Instant::now();
            println!("{:.2}%", (m * 100) as f64 / CHECK_UP_TO as f64)
        }
    }
    println!(
        "Program checked up to {CHECK_UP_TO} in {:.2} seconds.",
        start_time.elapsed().as_secs_f64()
    );
}
