use binary_search::{Direction, binary_search};
use rayon::prelude::*;
use rug::{Complete, Integer};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{Mutex, mpsc};
use tokio::task::JoinHandle;

const CHECK_UP_TO: usize = 4000;

#[inline]
fn next_m(prev: &mut [Integer; CHECK_UP_TO + 1], m: usize) {
    let mut last: Integer = Integer::from(0);
    for n in 1..=m {
        let new_last = unsafe { prev.get_unchecked(n).clone() };
        unsafe { *prev.get_unchecked_mut(n) = n * (prev.get_unchecked(n).clone() + last) };
        last = new_last;
    }
}

struct SurjectiveGenerator {
    _gen_handle: JoinHandle<()>,
    rx: mpsc::Receiver<([Integer; CHECK_UP_TO + 1], usize)>,
}

fn surjective_base_case() -> [Integer; CHECK_UP_TO + 1] {
    let mut surjective: [Integer; CHECK_UP_TO + 1] = std::array::from_fn(|_| Integer::from(0));
    surjective[1] = Integer::from(1);
    surjective
}

impl SurjectiveGenerator {
    fn new() -> Self {
        let (tx, rx) = mpsc::channel(1025);
        let thread_tx = tx.clone();
        let gen_handle = tokio::task::spawn(async move {
            let mut current = surjective_base_case();
            for i in 2..=CHECK_UP_TO {
                next_m(&mut current, i);
                let sent_current = current.clone();
                thread_tx.send((sent_current, i)).await.unwrap();
            }
            println!("gen done");
        });
        SurjectiveGenerator {
            _gen_handle: gen_handle,
            rx,
        }
    }
    async fn get_next(&mut self) -> ([Integer; CHECK_UP_TO + 1], usize) {
        self.rx.recv().await.unwrap()
    }
}

#[tokio::main]
async fn main() {
    let start_time = Instant::now();
    let sur_gen = Arc::new(Mutex::new(SurjectiveGenerator::new()));
    let mut handles = Vec::new();
    for _ in 2..=CHECK_UP_TO {
        let new_gen = Arc::clone(&sur_gen);
        let handle = tokio::task::spawn(async move {
            let (surjective, m) = new_gen.lock().await.get_next().await;
            let solution = (2..=m).into_par_iter().find_map_any(|z| unsafe {
                let x = z - 1;
                let right_side = surjective.get_unchecked(z);
                let x_side = surjective.get_unchecked(x);
                let change_point = binary_search((1, ()), (m - 1, ()), |i| {
                    if surjective.get_unchecked(i) <= surjective.get_unchecked(i + 1) {
                        Direction::Low(())
                    } else {
                        Direction::High(())
                    }
                })
                .0
                .0 + 1;
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
            } else {
                // println!("nothing for {m}");
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.await.unwrap();
    }
    println!(
        "Program checked up to {CHECK_UP_TO} in {:.2} seconds.",
        start_time.elapsed().as_secs_f64()
    );
}
