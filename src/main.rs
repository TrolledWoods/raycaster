#![feature(array_methods)]

use minifb::{Key, Window, WindowOptions};
mod raycast;
mod iget;
use iget::SignedIndex;

use raycast::*;

fn main() {
    let mut buffer: Vec<u32> = Vec::new();

	let mut world = [
		b"###################",
		b"#.................#",
		b"#.....##......##..#",
		b"#..........#...#..#",
		b"#..........#...#..#",
		b"#..#.......#......#",
		b"#..#..............#",
		b"#..#....######....#",
		b"#.................#",
		b"#......##....##...#",
		b"###################",
	];

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

	let mut frame_rate = [0f32; 10];
	let mut frame_rate_index = 0;
	let mut last_frame_time = 1.0;

	let mut column_rays = Vec::new();
    while window.is_open() && !window.is_key_down(Key::F4) {
		let (width, height) = window.get_size();

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

		column_rays.resize(width, (1000000000000.0, ()));
		for x in 0..width {
			let fx = (x as f32 / width as f32 - 0.5);

			column_rays[x] = raycast(Raycast {
					x: player_x,
					y: player_y,
					dx: dx + dy * fx,
					dy: dy - dx * fx,
					.. Default::default()
				},
				|x, y| if
					x >= 0 && y >= 0 &&
					(y as usize) < world.len() && (x as usize) < world[0].len()
				{
					if world[y as usize][x as usize] == b'#' {
						Some(())
					} else {
						None
					}
				} else {
					None
				},
			).unwrap_or((100000000000.0, ()));
		}


		for y in 0..height {
			let fy = 2.0 * (y as f32 / height as f32 - 0.5).abs();

			for x in 0..width {
				let (dist, _) = column_rays[x];
				buffer[y * width + x] = if fy < 1.0 / dist {
					((1.0 / (dist * dist * 0.25)).min(1.0) * 255.0) as u32
				} else {
					0
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
