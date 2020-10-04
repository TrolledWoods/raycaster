#![feature(array_methods)]

use minifb::{Key, Window, WindowOptions};
mod raycast;
mod iget;
use iget::SignedIndex;

use raycast::*;

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

fn main() {
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

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
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

	let mut player_x = 5.0;
	let mut player_y = 5.0;

	let mut player_rot: f32 = 0.0;

    while window.is_open() && !window.is_key_down(Key::F4) {
		if window.is_key_down(Key::Right) {
			player_rot -= 0.1;
		}
		if window.is_key_down(Key::Left) {
			player_rot += 0.1;
		}

		let dx = player_rot.cos();
		let dy = player_rot.sin();

		let player_speed = 0.1;
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

		for x in 0..WIDTH {
			let fx = (x as f32 / WIDTH as f32 - 0.5);

			match raycast(Raycast {
				x: player_x,
				y: player_y,
				dx: dx + dy * fx,
				dy: dy - dx * fx,
				.. Default::default()
			}, |x, y| match *world.as_slice().iget(y)?.iget(x)? == b'#' { true => Some(()), false => None }) {
				Some((dist, ())) => {
					for y in 0..HEIGHT {
						let fy = 2.0 * (y as f32 / HEIGHT as f32 - 0.5).abs();

						buffer[y * WIDTH + x] = if fy < 1.0 / (1.0 + dist) { ((1.0 / (1.0 + dist)) * 256.0) as u32 } else { 0 };
					}
				}
				None => for y in 0..HEIGHT {
					buffer[y * WIDTH + x] = 0;
				}
			}
		}

        window
            .update_with_buffer(&buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}
