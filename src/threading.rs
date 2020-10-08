use std::thread::{spawn, sleep, JoinHandle};
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use crate::world::World;
use crate::texture::Textures;
use crate::raycast::{raycast, Raycast};
use crate::render::ImageColumn;
use crate::texture::*;
use crate::{Vec2, Mat2};

// TODO: It's weird to have rendering in the threading file, so,
// either; make this more generic and move the rendering part somewhere else
// or    ; include the thread pool in the renderer.

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
	hit_data_cache: Vec<HitData>,
	floor_gfx_cache: Vec<FloorGfx>,
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
				let mut floor_gfx = Vec::new();
				while shared.keep_running.load(Ordering::SeqCst) {
					let mut lock = shared.work.lock().unwrap();
					if !lock.1.is_empty() {
						let work = lock.1.pop().unwrap();
						lock.0 += 1;
						std::mem::drop(lock);

						unsafe { run_work(work, &mut hits, &mut floor_gfx); }

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
			hit_data_cache: Vec::new(),
			floor_gfx_cache: Vec::new(),
		}
	}

	pub fn raycast_scene(
		&mut self,
		world: &World, textures: &Textures,
		cam_pos: Vec2, cam_matrix: Mat2,
		width: usize, height: usize, buffer: &mut [u32],
		aspect: f32,
	) {
		assert_eq!(width * height, buffer.len());

		if height == 0 {
			return;
		}

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

		while let Some(work) = {
			let value = self.shared.work.lock().unwrap().1.pop();
			value
		} {
			unsafe { run_work(work, &mut self.hit_data_cache, &mut self.floor_gfx_cache); }
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
	size: f32,
	uv: f32,
	texture_id: Texture,
	y_pos: f32,
}

struct FloorGfx {
	from_dist: f32,
	to_dist: f32,
	texture_id: Texture,
	from_uv: Vec2,
	to_uv: Vec2,
}

unsafe fn run_work(work: RaycastWork, hits: &mut Vec<HitData>, floor_gfx: &mut Vec<FloorGfx>) {
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
		let fx = (0.5 - (x + x_offset) as f32 / stride as f32) / aspect;
		let offset = cam_matrix * Vec2::new(fx, 1.0);

		let inv_cam_matrix = crate::inverse_mat2(cam_matrix);

		hits.clear();
		floor_gfx.clear();

		let mut prev: Option<FloorGfx> = None;
		raycast(Raycast {
				x: cam_pos.x,
				y: cam_pos.y,
				dx: offset.x,
				dy: offset.y,
				.. Default::default()
			},
			|dist, x, y, off_x, off_y, pos| {
				if let Some(mut prev) = prev.take() {
					prev.to_dist = dist;
					prev.to_uv = pos;
					floor_gfx.push(prev);
				}

				let tile = world.tiles.get(x, y);
				let should_continue = match tile {
					Some(tile) => {
						prev = Some(FloorGfx {
							from_dist: dist,
							to_dist: 0.0,
							texture_id: tile.floor_gfx,
							from_uv: pos,
							to_uv: Vec2::zero(),
						});

						match tile.get_graphics() {
							Some(graphics) => {
								hits.push(HitData {
									dist,
									uv: off_x + off_y, 
									texture_id: graphics.texture,
									size: 1.0,
									y_pos: 0.5,
								});
								graphics.is_transparent
							}
							None => {
								for &sprite_id in tile.sprites_inside.iter() {
									let entity = world.get_sprite(sprite_id).unwrap();

									let rel_entity_pos = inv_cam_matrix * (entity.pos() - cam_pos);
									let hit_x = 0.5 + (rel_entity_pos.x - fx * rel_entity_pos.y) / entity.size();
									if hit_x >= 0.0 && hit_x < 1.0 {
										hits.push(HitData {
											dist: rel_entity_pos.y,
											uv: hit_x,
											texture_id: entity.texture,
											size: entity.size(),
											y_pos: entity.y_pos,
										});
									}
								}

								true
							}
						}
					}
					None => {
						prev = None;
						false
					}
				};
				should_continue
			},
		);

		// Sort the graphics by distance
		hits.sort_unstable_by(
			|a, b| a.dist.partial_cmp(&b.dist).unwrap()
		);

		let mut column = ImageColumn::from_raw(buffer.add(x), stride, height);

		for &FloorGfx { from_dist, to_dist, texture_id, from_uv, to_uv } in floor_gfx.iter().take(0) {
			let from_dist_size = 1.0f32 / (0.0000001 + from_dist);
			let to_dist_size   = 1.0f32 / (0.0000001 + to_dist);

			column.draw_uv_row(textures.get(texture_id),
				to_uv, from_uv,
				0.5 + to_dist_size * 0.5, 0.5 + from_dist_size * 0.5,
				1.0 / (1.0 + from_dist * from_dist * 0.1),
			);
		}

		for hit in hits.iter().rev() {
			let dist_size = 1.0f32 / (0.0000001 + hit.dist);
			let texture = textures.get(hit.texture_id);
			column.draw_partial_image(texture, 
				((hit.uv * texture.height() as f32) as u32).clamp(0, texture.height() - 1),
				0.0, 1.0,
				0.5 - dist_size * 0.5 + dist_size * (hit.y_pos * (1.0 - hit.size)),
				0.5 - dist_size * 0.5 + dist_size * (hit.y_pos * (1.0 - hit.size) + hit.size),
				1.0 / (1.0 + hit.dist * hit.dist * 0.4),
			);
		}
	}
}
