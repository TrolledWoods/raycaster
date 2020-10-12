use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn, JoinHandle};
use std::time::Duration;

use crate::raycast::{raycast, Raycast};
use crate::render::ImageColumn;
use crate::texture::{Textures, VerticalImage};
use crate::world::World;
use crate::{Mat2, Vec2};

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
    world_time: f32,
}

// SAFETY: This is safe because we are not using thread local storage or anything.
unsafe impl Send for RaycastWork {}

struct SharedData {
    work: Mutex<(u32, Vec<RaycastWork>)>,
    keep_running: AtomicBool,
}

pub struct ThreadPool<'a> {
    threads: Vec<JoinHandle<()>>,
    shared: Arc<SharedData>,
    hit_data_cache: Vec<HitData<'a>>,
}

impl ThreadPool<'_> {
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

                        unsafe {
                            run_work(work, &mut hits);
                        }

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
        }
    }

    pub fn raycast_scene(
        &mut self,
        world: &World,
        textures: &Textures,
        cam_pos: Vec2,
        cam_matrix: Mat2,
        width: usize,
        height: usize,
        buffer: &mut [u32],
        aspect: f32,
        world_time: f32,
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
                world_time,
            });
        }

        while let Some(work) = {
            let value = self.shared.work.lock().unwrap().1.pop();
            value
        } {
            unsafe {
                run_work(work, &mut self.hit_data_cache);
            }
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
struct HitData<'a> {
    dist: f32,
    size: f32,
    uv: f32,
    image: &'a VerticalImage,
    y_pos: f32,
}

unsafe fn run_work(work: RaycastWork, hits: &mut Vec<HitData>) {
    let RaycastWork {
        world,
        textures,
        buffer,
        stride,
        width,
        height,
        x_offset,
        cam_matrix,
        cam_pos,
        aspect,
        world_time,
    } = work;

    // If the RaycastWork is valid, this should be valid too!
    let world = &*world;
    let textures = &*textures;

    for x in 0..width {
        let fx = (0.5 - (x + x_offset) as f32 / stride as f32) / aspect;
        let offset = cam_matrix * Vec2::new(fx, 1.0);

        let inv_cam_matrix = crate::inverse_mat2(cam_matrix);

        hits.clear();

        raycast(
            Raycast {
                x: cam_pos.x,
                y: cam_pos.y,
                dx: offset.x,
                dy: offset.y,
                ..Default::default()
            },
            |dist, x, y, off_x, off_y, _pos| {
                let tile = world.tiles.get(x, y);
                let should_continue = match tile {
                    Some(tile) => match tile.get_graphics() {
                        Some(graphics) => {
                            hits.push(HitData {
                                dist,
                                uv: off_x + off_y,
                                image: textures.get_anim(&graphics.texture, world_time),
                                size: 1.0,
                                y_pos: 0.5,
                            });
                            graphics.is_transparent
                        }
                        None => {
                            for &sprite_id in tile.sprites_inside.iter() {
                                let entity = world.get_sprite(sprite_id).unwrap();

                                let rel_entity_pos = inv_cam_matrix * (entity.pos() - cam_pos);
                                let hit_x = 0.5
                                    + (rel_entity_pos.x - fx * rel_entity_pos.y) / entity.size();
                                if hit_x >= 0.0 && hit_x < 1.0 {
                                    hits.push(HitData {
                                        dist: rel_entity_pos.y,
                                        uv: hit_x,
                                        image: textures.get(entity.texture),
                                        size: entity.size(),
                                        y_pos: entity.y_pos,
                                    });
                                }
                            }

                            true
                        }
                    },
                    None => false,
                };
                should_continue
            },
        );

        // Sort the graphics by distance
        hits.sort_unstable_by(|a, b| a.dist.partial_cmp(&b.dist).unwrap());

        let mut column = ImageColumn::from_raw(buffer.add(x), stride, height);

        for hit in hits.iter().rev() {
            let dist_size = 1.0f32 / (0.0000001 + hit.dist);
            column.draw_partial_image(
                hit.image,
                ((hit.uv * hit.image.width() as f32) as usize).clamp(0, hit.image.width() - 1),
                0.0,
                1.0,
                0.5 - dist_size * 0.5 + dist_size * (hit.y_pos * (1.0 - hit.size)),
                0.5 - dist_size * 0.5 + dist_size * (hit.y_pos * (1.0 - hit.size) + hit.size),
                1.0 / (1.0 + hit.dist * hit.dist * 0.2),
            );
        }
    }
}
