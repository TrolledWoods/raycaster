#![feature(array_methods)]
#![feature(clamp)]

use minifb::{Key, Window, WindowOptions};
mod raycast;
mod world;
// mod threading;

use raycast::*;

fn main() {
	let image = image::open("assets/wall.png").unwrap().into_rgba();

    let mut buffer: Vec<u32> = Vec::new();

	let world = world::World::new();

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

    window.limit_update_rate(Some(std::time::Duration::from_secs_f32(1.0 / 30.0)));

	let mut player_x = 5.0;
	let mut player_y = 5.0;

	let mut player_rot: f32 = 0.0;

	let mut frame_rate = [0f32; 50];
	let mut frame_rate_index = 0;
	let mut last_frame_time = 1.0;
    while window.is_open() && !window.is_key_down(Key::F4) {
		let (width, height) = window.get_size();
		let aspect = height as f32 / width as f32;

		buffer.resize(width * height, 0);

		let instant = std::time::Instant::now();

		if window.is_key_down(Key::Right) {
			player_rot -= 5.0 * last_frame_time;
		}
		if window.is_key_down(Key::Left) {
			player_rot += 5.0 * last_frame_time;
		}

		let dx = player_rot.cos();
		let dy = player_rot.sin();

		let player_speed = 4.0 * last_frame_time;
		if window.is_key_down(Key::A) {
			player_x -= dy * player_speed;
			player_y += dx * player_speed;
		}
		if window.is_key_down(Key::D) {
			player_x += dy * player_speed;
			player_y -= dx * player_speed;
		}
		if window.is_key_down(Key::W) {
			player_x += dx * player_speed;
			player_y += dy * player_speed;
		}
		if window.is_key_down(Key::S) {
			player_x -= dx * player_speed;
			player_y -= dy * player_speed;
		}

		for x in 0..width {
			let fx = (x as f32 / width as f32 - 0.5) / aspect;

			let mut size = 0.0;
			let mut inv_size = 0.0;
			let mut uv = 0.0;
			raycast(Raycast {
					x: player_x,
					y: player_y,
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

				buffer[y * width + x] = if 2.0 * fy.abs() < size {
					u32::from_le_bytes(image.get_pixel(
						(uv * 32.0).clamp(0.0, 31.0) as u32,
						((fy * inv_size + 0.5) * 32.0).clamp(0.0, 31.0) as u32
					).0)
				} else {
					0x101010
				};
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
}
