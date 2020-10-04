use std::thread::{spawn, sleep, JoinHandle};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::world::World;
use crate::texture::Textures;
use crate::raycast::{raycast, Raycast};

const SPLIT_SIZE: usize = 64;

/// A piece of work representing an area to raycast.
struct RaycastWork {
	world: *const World,
	textures: *const Textures,
	buffer: *mut u32,
	stride: usize,
	width: usize,
	height: usize,
	x_offset: usize,
	dx: f32,
	dy: f32,
	cam_x: f32,
	cam_y: f32,
	aspect: f32,
}

// SAFETY: This is safe because we are not using thread local storage or anything.
unsafe impl Send for RaycastWork {}

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
			work: Mutex::new((0, Vec::new())),
			keep_running: AtomicBool::new(true),
		});

		for _ in 0..n_threads {
			let shared = shared.clone();
			threads.push(spawn(move || {
				while shared.keep_running.load(Ordering::SeqCst) {
					let mut lock = shared.work.lock().unwrap();
					if !lock.1.is_empty() {
						let work = lock.1.pop().unwrap();
						lock.0 += 1;
						std::mem::drop(lock);

						unsafe { run_work(work); }

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
			threads,
			shared,
		}
	}

	pub fn raycast_scene(
		&self,
		world: &World, textures: &Textures,
		cam_x: f32, cam_y: f32, cam_rot: f32,
		width: usize, height: usize, buffer: &mut [u32],
		aspect: f32,
	) {
		assert_eq!(width * height, buffer.len());

		let dx = cam_rot.cos();
		let dy = cam_rot.sin();

		for (i, chunk) in buffer[0..width].chunks_mut(SPLIT_SIZE).enumerate() {
			self.shared.work.lock().unwrap().1.push(RaycastWork {
				world,
				textures,
				buffer: chunk.as_mut_ptr(),
				stride: width,
				width: chunk.len(),
				height,
				cam_x,
				cam_y,
				x_offset: i * SPLIT_SIZE,
				dx,
				dy,
				aspect,
			});
		}

		while let Some(work) = {
			let value = self.shared.work.lock().unwrap().1.pop();
			value
		} {
			unsafe { run_work(work); }
		}

		// While work is still being performed, wait
		while self.shared.work.lock().unwrap().0 > 0 {
			sleep(Duration::from_nanos(200));
		}
	}

	pub fn join(self) {
		self.shared.keep_running.store(false, Ordering::SeqCst);
		for thread in self.threads {
			let _ = thread.join();
		}
	}
}

unsafe fn run_work(work: RaycastWork) {
	let RaycastWork {
		world, textures, buffer, stride, width, height,
		x_offset, dx, dy,
		cam_x, cam_y,
		aspect,
	} = work;

	// If the RaycastWork is valid, this should be valid too!
	let world = &*world;
	let textures = &*textures;

	for x in 0..width {
		let fx = ((x + x_offset) as f32 / stride as f32 - 0.5) / aspect;

		let mut size = 0.0;
		let mut inv_size = 0.0;
		let mut uv = 0.0;
		raycast(Raycast {
				x: cam_x,
				y: cam_y,
				dx: dx + dy * fx,
				dy: dy - dx * fx,
				.. Default::default()
			},
			|dist, x, y, off_x, off_y| if world.get(x, y) == Some(b'#') {
				size = 1.0 / dist;
				inv_size = dist;
				uv = off_x + off_y;
				false
			} else {
				true
			},
		);

		for y in 0..height {
			let fy = y as f32 / height as f32 - 0.5;

			*buffer.add(y * stride + x) = if 2.0 * fy.abs() < size {
				u32::from_le_bytes(textures.wall.get_pixel(
					(uv * 32.0).clamp(0.0, 31.0) as u32,
					((fy * inv_size + 0.5) * 32.0).clamp(0.0, 31.0) as u32
				).0)
			} else {
				0x101010
			};
		}
	}
}
