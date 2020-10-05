use std::thread::{spawn, sleep, JoinHandle};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::world::World;
use crate::texture::Textures;
use crate::raycast::{raycast, Raycast};
use crate::render::ImageColumn;
use crate::{Vec2, Mat2};

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
	cam_matrix: Mat2,
	cam_pos: Vec2,
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
				let mut hits = Vec::new();
				while shared.keep_running.load(Ordering::SeqCst) {
					let mut lock = shared.work.lock().unwrap();
					if !lock.1.is_empty() {
						let work = lock.1.pop().unwrap();
						lock.0 += 1;
						std::mem::drop(lock);

						unsafe { run_work(work, &mut hits); }

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
		cam_pos: Vec2, cam_matrix: Mat2,
		width: usize, height: usize, buffer: &mut [u32],
		aspect: f32,
	) {
		assert_eq!(width * height, buffer.len());

		for (i, chunk) in buffer[0..width].chunks_mut(SPLIT_SIZE).enumerate() {
			self.shared.work.lock().unwrap().1.push(RaycastWork {
				world,
				textures,
				buffer: chunk.as_mut_ptr(),
				stride: width,
				width: chunk.len(),
				height,
				cam_pos,
				cam_matrix,
				x_offset: i * SPLIT_SIZE,
				aspect,
			});
		}

		let mut hits = Vec::new();
		while let Some(work) = {
			let value = self.shared.work.lock().unwrap().1.pop();
			value
		} {
			unsafe { run_work(work, &mut hits); }
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

#[derive(Clone, Copy)]
struct HitData {
	dist: f32,
	uv: f32,
	texture_id: u16,
}

unsafe fn run_work(work: RaycastWork, hits: &mut Vec<HitData>) {
	let RaycastWork {
		world, textures, buffer, stride, width, height,
		x_offset,
		cam_matrix,
		cam_pos,
		aspect,
	} = work;

	// If the RaycastWork is valid, this should be valid too!
	let world = &*world;
	let textures = &*textures;

	for x in 0..width {
		let fx = ((x + x_offset) as f32 / stride as f32 - 0.5) / aspect;
		let offset = cam_matrix * Vec2::new(-fx, 1.0);

		hits.clear();
		let mut prev = None;
		raycast(Raycast {
				x: cam_pos.x,
				y: cam_pos.y,
				dx: offset.x,
				dy: offset.y,
				.. Default::default()
			},
			|dist, x, y, off_x, off_y| {
				let tile = world.tiles.get(x, y);
				let should_continue = match tile {
					Some(tile) => {
						match tile.get_graphics() {
							Some(graphics) => {
								hits.push(HitData {
									dist,
									uv: off_x + off_y, 
									texture_id: graphics.texture,
								});
								graphics.is_transparent
							}
							None => true
						}
					}
					None => true
				};
				prev = tile;
				should_continue
			},
		);

		let mut column = ImageColumn::from_raw(buffer.add(x), stride, height);
		for hit in hits.iter().rev() {
			let size = 1.0f32 / (0.0000001 + hit.dist);
			column.draw_partial_image(textures.get(hit.texture_id), 
				(hit.uv * 32.0).clamp(0.0, 31.0) as u32,
				0.0, 1.0,
				0.5 - size / 2.0,
				0.5 + size / 2.0,
				1.0 / (hit.dist * hit.dist * 0.1),
			);
		}
	}
}
