use std::thread::{spawn, sleep, JoinHandle};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// A piece of work representing an area to raycast.
struct RaycastWork {
	world: *const World,
	textures: *const Textures,
	pointer: *mut u32,
	stride: usize,
	width: usize,
	height: usize,
}

struct SharedData {
	work: Mutex<(u32, Vec<RaycastWork>)>,
	keep_running: AtomicBool,
}

pub struct ThreadPool {
	threads: Vec<JoinHandle<()>>,
	shared: Arc<SharedData>,
}

impl ThreadPool {
	pub fn new(n_threads: usize) -> Self {
		let mut threads = Vec::new();

		let shared = Arc::new(SharedData {
			work: Mutex::new(Vec::new()),
			keep_running: AtomicBool::new(true),
		});

		for _ in 0..n_threads {
			let shared = shared.clone();
			threads.push(spawn(move || {
				while shared.keep_running.load(Ordering::SeqCst) {
					let mut lock = shared.work.lock().unwrap();
					if !lock.1.is_empty() {
						lock.0 += 1;
						std::mem::drop(lock);

						// Do work

						let mut lock = shared.work.lock().unwrap();
						lock.0 -= 1;
						std::mem::drop(lock);
					} else {
						std::mem::drop(lock);
						sleep(Duration::from_nanos(10));
					}
				}
			}));
		}

		Self {
		}
	}

	pub fn raycast_scene(world: &World, textures: &Textures, pointer: 

	pub fn join(self) {
		self.shared.keep_running.store(false, Ordering::SeqCst);
		for thread in self.threads {
			let _ = thread.join();
		}
	}
}
