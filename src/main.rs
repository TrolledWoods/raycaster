#![feature(array_methods)]
#![feature(clamp)]

use minifb::{Key, Window, WindowOptions};
mod raycast;
mod world;
mod texture;
mod threading;
mod render;
mod float_range;
mod random;

type Vec2 = vek::vec::repr_simd::Vec2<f32>;
type Mat2 = vek::mat::repr_simd::column_major::Mat2<f32>;

fn main() {
	let textures = texture::Textures::create(vec![
		image::open("assets/wall.png").unwrap().into_rgba(),
		image::open("assets/window.png").unwrap().into_rgba(),
		image::open("assets/evil.png").unwrap().into_rgba(),
		image::open("assets/rick.png").unwrap().into_rgba(),
		image::open("assets/floor.png").unwrap().into_rgba(),
	]);

    let mut buffer: Vec<u32> = Vec::new();

	let mut random = random::Random::new();
	let (player_id, mut world) = world::generate::WorldGenerator::new(100, 100)
		.generate(&mut random, Vec2::one() * 2.5);

	world.to_image("output_maze.png");	

    let mut window = Window::new(
        "Raycaster",
        640,
        480,
        WindowOptions {
			resize: true,
			.. WindowOptions::default()
		},
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 40.0)));

	let mut cam_pos = Vec2::new(5.0, 5.0);
	let mut cam_matrix = Mat2::zero();
	let mut inv_cam_matrix = Mat2::zero();

	let mut frame_rate = [0f32; 50];
	let mut frame_rate_index = 0;
	let mut last_frame_time = 1.0;

	let mut thread_pool = threading::ThreadPool::new(4);
    while window.is_open() && !window.is_key_down(Key::F4) {
		let (width, height) = window.get_size();
		let aspect = height as f32 / width as f32;

		buffer.resize(width * height, 0);

		let instant = std::time::Instant::now();

		if let Some(player) = world.get_entity_mut(player_id) {
			if window.is_key_down(Key::Right) {
				player.rot += 5.0 * last_frame_time;
			}
			if window.is_key_down(Key::Left) {
				player.rot -= 5.0 * last_frame_time;
			}

			cam_matrix = Mat2::identity().rotated_z(player.rot);
			inv_cam_matrix = Mat2::identity().rotated_z(-player.rot);

			let player_speed = 0.2;
			if window.is_key_down(Key::A) {
				player.vel += cam_matrix * Vec2::right() * player_speed;
			}
			if window.is_key_down(Key::D) {
				player.vel += cam_matrix * Vec2::left() * player_speed;
			}
			if window.is_key_down(Key::W) {
				player.vel += cam_matrix * Vec2::up() * player_speed;
			}
			if window.is_key_down(Key::S) {
				player.vel += cam_matrix * Vec2::down() * player_speed;
			}
		}

		world.simulate_physics(last_frame_time);

		if let Some(player) = world.get_entity(player_id) {
			cam_pos = player.pos;
		}

		for val in buffer.iter_mut() { *val = 0; }
		thread_pool.raycast_scene(
			&world, &textures,
			cam_pos, cam_matrix,
			width, height, &mut buffer,
			aspect
		);

		for (_, entity) in world.entities() {
			let diff = entity.pos - cam_pos;
			let inverted = inv_cam_matrix * diff;

			if inverted.y >= 0.05 {
				let mut draw = true;
				raycast::raycast(raycast::Raycast {
					x: cam_pos.x,
					y: cam_pos.y,
					dx: diff.x,
					dy: diff.y,
					max_distance: 1.0,
				}, |_, x, y, _, _, _| if world.tiles.tile_is_colliding(x, y) {
					draw = false;
					false
				} else {
					true
				});

				if draw {
					// &textures.textures[entity.texture]
				}
			}
		}

		frame_rate[frame_rate_index] = instant.elapsed().as_secs_f32();

        window
            .update_with_buffer(&buffer, width, height)
            .unwrap();

		last_frame_time = instant.elapsed().as_secs_f32();
		frame_rate_index += 1;
		if frame_rate_index >= frame_rate.len() {
			frame_rate_index = 0;
			let average: f32 = frame_rate.iter().sum::<f32>() / frame_rate.len() as f32;
			println!("{} seconds / {} fps (if no fps cap was applied)", average, 1.0 / average);
		}
    }

	thread_pool.join();
}

pub fn inverse_mat2(mat: Mat2) -> Mat2 {
	let [a, c, b, d] = mat.into_col_array();
	Mat2::new(d, -b, -c, a) * mat.determinant()
}
