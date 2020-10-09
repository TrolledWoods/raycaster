#![feature(array_methods)]
#![feature(clamp)]

use minifb::{Key, Window, WindowOptions};
mod alloc;
mod float_range;
mod random;
mod raycast;
mod render;
mod texture;
mod threading;
mod world;

type Vec2 = vek::vec::repr_simd::Vec2<f32>;
type Mat2 = vek::mat::repr_simd::column_major::Mat2<f32>;

fn main() {
	let textures = texture::Textures::new().unwrap();

	let mut buffer: Vec<u32> = Vec::new();

	let mut random = random::Random::new();
	let (player_id, mut world) =
		world::generate::WorldGenerator::new(100, 100, "assets\\room_prefabs.txt")
			.unwrap()
			.generate(&mut random, Vec2::one() * 102.5);

	world.to_image("debug_maze.png");

	let mut window = Window::new(
		"Raycaster",
		640,
		480,
		WindowOptions {
			resize: true,
			..WindowOptions::default()
		},
	)
	.unwrap_or_else(|e| {
		panic!("{}", e);
	});

	window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 40.0)));

	let mut cam_pos = Vec2::new(5.0, 5.0);
	let mut cam_matrix = Mat2::zero();

	let mut frame_rate = [0f32; 50];
	let mut frame_rate_index = 0;
	let mut last_frame_time = 1.0;

	let mut elapsed_time = 0.0;

	let mut thread_pool = threading::ThreadPool::new(4);
	while window.is_open() && !window.is_key_down(Key::F4) {
		let (width, height) = window.get_size();
		let aspect = height as f32 / width as f32;

		buffer.resize(width * height, 0);

		let instant = std::time::Instant::now();

		if let Some(player) = world.entities.get_mut(player_id) {
			if window.is_key_down(Key::Right) {
				player.rot += 5.0 * last_frame_time;
			}
			if window.is_key_down(Key::Left) {
				player.rot -= 5.0 * last_frame_time;
			}

			cam_matrix = Mat2::identity().rotated_z(player.rot);

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

		world.simulate_physics(last_frame_time, elapsed_time);

		if let Some(player) = world.entities.get(player_id) {
			cam_pos = player.pos;
		}

		for val in buffer.iter_mut() {
			*val = 0;
		}
		thread_pool.raycast_scene(
			&world,
			&textures,
			cam_pos,
			cam_matrix,
			width,
			height,
			&mut buffer,
			aspect,
			elapsed_time,
		);

		frame_rate[frame_rate_index] = instant.elapsed().as_secs_f32();

		window.update_with_buffer(&buffer, width, height).unwrap();

		last_frame_time = instant.elapsed().as_secs_f32();
		elapsed_time += last_frame_time;
		frame_rate_index += 1;
		if frame_rate_index >= frame_rate.len() {
			frame_rate_index = 0;
			let average: f32 = frame_rate.iter().sum::<f32>() / frame_rate.len() as f32;
			println!(
				"{} seconds / {} fps (if no fps cap was applied)",
				average,
				1.0 / average
			);
			println!(
				"{} allocations",
				crate::alloc::ALLOCATOR
					.allocations_count
					.load(std::sync::atomic::Ordering::Relaxed)
			);
		}
	}

	thread_pool.join();
}

pub fn inverse_mat2(mat: Mat2) -> Mat2 {
	let [a, c, b, d] = mat.into_col_array();
	Mat2::new(d, -b, -c, a) * mat.determinant()
}
