use std::cell::RefCell;
use std::collections::HashMap;

pub const BENCHMARK: bool = false;

thread_local!(static BENCH_MAP: RefCell<HashMap<&'static str, u128>> = RefCell::new(HashMap::new()));

#[allow(unused)]
pub struct Bench {
	pub func_name: &'static str,
	pub file: &'static str,
	pub line: u32,
	pub column: u32,
	pub instant: std::time::Instant,
}

impl std::ops::Drop for Bench {
	fn drop(&mut self) {
		let ns = self.instant.elapsed().as_nanos();
		crate::benchmark::BENCH_MAP.with(|map| {
			*map.borrow_mut().entry(&self.func_name).or_insert(0) += ns;
		});
	}
}

macro_rules! bench {
	($name:expr) => {
		let _bench_expr = if crate::benchmark::BENCHMARK {
			Some(crate::benchmark::Bench {
				func_name: $name,
				file: file!(),
				line: line!(),
				column: column!(),
				instant: std::time::Instant::now(),
			})
		} else {
			None
		};
	}
}

pub fn print_bench_results() {
	if BENCHMARK {
		println!("-- BENCHMARKING RESULTS --");
		BENCH_MAP.with(|map| {
			for (key, value) in map.borrow().iter() {
				println!("{}: {}", key, value);
			}
		});
	}
}
