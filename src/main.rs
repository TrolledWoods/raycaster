#![feature(array_methods)]
#![feature(clamp)]

use minifb::{Key, Window, WindowOptions};
mod raycast;
mod world;
mod texture;
mod threading;
mod render;
mod float_range;

use world::Entity;

fn main() {
	let textures = texture::Textures::create(vec![
		image::open("assets/wall.png").unwrap().into_rgba(),
		image::open("assets/window.png").unwrap().into_rgba(),
	]);

    let mut buffer: Vec<u32> = Vec::new();

	let mut world = world::World::new(&[
		b"###################",
		b"#                 #",
		b"#  #############  #",
		b"#  #    #         #",
		b"#  #    o         #",
		b"#       #         #",
		b"#  #####  # #   # #",
		b"#  #   # #####o## #",
		b"#  # # # #  #   # #",
		b"#    #   #        #",
		b"###################",
	]);
	let player_id = world.insert_entity(Entity::new(5.0, 5.0, 0.1));

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

	let mut cam_x = 5.0;
	let mut cam_y = 5.0;

	let mut cam_rot: f32 = 0.0;

	let mut frame_rate = [0f32; 50];
	let mut frame_rate_index = 0;
	let mut last_frame_time = 1.0;

	let thread_pool = threading::ThreadPool::new(5);
    while window.is_open() && !window.is_key_down(Key::F4) {
		let (width, height) = window.get_size();
		let aspect = height as f32 / width as f32;

		buffer.resize(width * height, 0);

		let instant = std::time::Instant::now();

		if let Some(player) = world.get_entity_mut(player_id) {
			if window.is_key_down(Key::Right) {
				player.rot -= 5.0 * last_frame_time;
			}
			if window.is_key_down(Key::Left) {
				player.rot += 5.0 * last_frame_time;
			}

			let dx = player.rot.cos();
			let dy = player.rot.sin();

			let player_speed = 0.2;
			if window.is_key_down(Key::A) {
				player.vx -= dy * player_speed;
				player.vy += dx * player_speed;
			}
			if window.is_key_down(Key::D) {
				player.vx += dy * player_speed;
				player.vy -= dx * player_speed;
			}
			if window.is_key_down(Key::W) {
				player.vx += dx * player_speed;
				player.vy += dy * player_speed;
			}
			if window.is_key_down(Key::S) {
				player.vx -= dx * player_speed;
				player.vy -= dy * player_speed;
			}
		}

		world.simulate_physics(last_frame_time);

		if let Some(player) = world.get_entity(player_id) {
			cam_x = player.x;
			cam_y = player.y;
			cam_rot = player.rot;
		}

		for val in buffer.iter_mut() { *val = 0; }
		thread_pool.raycast_scene(
			&world, &textures,
			cam_x, cam_y, cam_rot,
			width, height, &mut buffer,
			aspect
		);

		// for x in 0..width {
		// 	let fx = (x as f32 / width as f32 - 0.5) / aspect;

		// 	let mut size = 0.0;
		// 	let mut inv_size = 0.0;
		// 	let mut uv = 0.0;
		// 	raycast(Raycast {
		// 			x: player_x,
		// 			y: player_y,
		// 			dx: dx + dy * fx,
		// 			dy: dy - dx * fx,
		// 			.. Default::default()
		// 		},
		// 		|dist, x, y, off_x, off_y| if world.get(x, y) == Some(b'#') {
		// 			size = 1.0 / dist;
		// 			inv_size = dist;
		// 			uv = off_x + off_y;
		// 			false
		// 		} else {
		// 			true
		// 		},
		// 	);

		// 	for y in 0..height {
		// 		let fy = y as f32 / height as f32 - 0.5;

		// 		buffer[y * width + x] = if 2.0 * fy.abs() < size {
		// 			u32::from_le_bytes(textures.wall.get_pixel(
		// 				(uv * 32.0).clamp(0.0, 31.0) as u32,
		// 				((fy * inv_size + 0.5) * 32.0).clamp(0.0, 31.0) as u32
		// 			).0)
		// 		} else {
		// 			0x101010
		// 		};
		// 	}
		// }

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
