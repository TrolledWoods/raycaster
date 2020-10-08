use std::collections::HashMap;
use std::num::NonZeroU32;
use super::{Tile, TileKind, World, Entity, EntityId, TileMap};
use crate::random::Random;
use crate::Vec2;
use crate::texture::Texture;

const ROOM_WIDTH:  usize = 4;
const ROOM_HEIGHT: usize = 4;

#[allow(non_upper_case_globals)]
const Fa: GenTile = GenTile::new(GenTileKind::Floor);
#[allow(non_upper_case_globals)]
const Wu: GenTile = GenTile::new(GenTileKind::Wall).change_if(GenTileKind::Floor, Direction::Up);
#[allow(non_upper_case_globals)]
const Wd: GenTile = GenTile::new(GenTileKind::Wall).change_if(GenTileKind::Floor, Direction::Down);
#[allow(non_upper_case_globals)]
const Wl: GenTile = GenTile::new(GenTileKind::Wall).change_if(GenTileKind::Floor, Direction::Left);
#[allow(non_upper_case_globals)]
const Wr: GenTile = GenTile::new(GenTileKind::Wall).change_if(GenTileKind::Floor, Direction::Right);
#[allow(non_upper_case_globals)]
const Wa: GenTile = GenTile::new(GenTileKind::Wall);
#[allow(non_upper_case_globals)]
const Ga: GenTile = GenTile::new(GenTileKind::Window);

#[derive(Clone)]
pub enum GenTileKind {
	Floor,
	Wall,
	Window,
}

#[derive(Clone)]
pub struct GenTile {
	pub kind: GenTileKind,
	pub change_if: Option<(GenTileKind, Direction)>,
}

impl GenTile {
	pub const fn new(kind: GenTileKind) -> Self {
		Self {
			kind,
			change_if: None,
		}
	}

	pub const fn change_if(mut self, kind: GenTileKind, dir: Direction) -> Self {
		self.change_if = Some((kind, dir));
		self
	}
}

pub struct WorldGenerator {
	n_rooms_width: usize,
	n_rooms_height: usize,
}

impl WorldGenerator {
	pub fn new(n_rooms_width: usize, n_rooms_height: usize) -> Self {
		Self {
			n_rooms_width,
			n_rooms_height,
		}
	}

	pub fn generate(&self, random: &mut Random, start: Vec2) -> (EntityId, World) {
		struct LooseEnd {
			from_x: isize,
			from_y: isize,
			direction: Direction,
		}

		let mut loose_ends = vec![
			LooseEnd {
				from_x: (start.x.floor() / ROOM_WIDTH as f32) as isize,
				from_y: (start.y.floor() / ROOM_HEIGHT as f32) as isize,
				direction: Direction::Left,
			},
			LooseEnd {
				from_x: (start.x.floor() / ROOM_WIDTH as f32) as isize,
				from_y: (start.y.floor() / ROOM_HEIGHT as f32) as isize,
				direction: Direction::Down,
			},
			LooseEnd {
				from_x: (start.x.floor() / ROOM_WIDTH as f32) as isize,
				from_y: (start.y.floor() / ROOM_HEIGHT as f32) as isize,
				direction: Direction::Up,
			},
			LooseEnd {
				from_x: (start.x.floor() / ROOM_WIDTH as f32) as isize,
				from_y: (start.y.floor() / ROOM_HEIGHT as f32) as isize,
				direction: Direction::Right,
			},
		];
		let mut rooms = Rooms::new(self.n_rooms_width, self.n_rooms_height);
		*rooms.get_mut((start.x.floor() / ROOM_WIDTH as f32) as isize, (start.y.floor() / ROOM_HEIGHT as f32) as isize).unwrap() =
			Some(Room::new(0, 0, 0));

		let room_prefabs = vec![
			RoomPrefab {
				chance: 5.0,
				tiles: vec![
					Wa, Wu, Wu, Wa,
					Wl, Fa, Fa, Wr,
					Wl, Fa, Fa, Wr,
					Wa, Wd, Wd, Wa,
				],
				n_rooms_width: 1,
				n_rooms_height: 1,
			},
			RoomPrefab {
				chance: 0.2,
				tiles: vec![
					Wa, Wu, Wu, Wa, Wa, Wu, Wu, Wa, Wa, Wu, Wu, Wa, Wa, Wu, Wu, Wa,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wa,
					Wa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wa,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wa, Fa, Fa, Fa, Fa, Fa, Ga, Fa, Ga, Fa, Ga, Fa, Fa, Fa, Fa, Wa,
					Wa, Fa, Fa, Fa, Fa, Fa, Ga, Fa, Ga, Fa, Ga, Fa, Fa, Fa, Fa, Wa,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wa, Fa, Fa, Fa, Fa, Fa, Ga, Fa, Ga, Fa, Ga, Fa, Fa, Fa, Fa, Wa,
					Wa, Fa, Fa, Fa, Fa, Fa, Ga, Fa, Ga, Fa, Ga, Fa, Fa, Fa, Fa, Wa,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wa,
					Wa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wa,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wl, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Fa, Wr,
					Wa, Wd, Wd, Wa, Wa, Wd, Wd, Wa, Wa, Wd, Wd, Wa, Wa, Wd, Wd, Wa,
				],
				n_rooms_width: 4,
				n_rooms_height: 4,
			},
			RoomPrefab {
				chance: 1.0,
				tiles: vec![
					Wa, Wu, Wu, Wa, Wa, Wu, Wu, Wa,
					Wl, Fa, Fa, Ga, Fa, Fa, Fa, Wr,
					Wl, Fa, Fa, Ga, Fa, Fa, Fa, Wr,
					Wa, Fa, Fa, Ga, Fa, Fa, Fa, Wa,
					Wa, Fa, Fa, Fa, Fa, Fa, Fa, Wa,
					Wl, Fa, Fa, Ga, Fa, Fa, Fa, Wr,
					Wl, Fa, Fa, Ga, Fa, Fa, Fa, Wr,
					Wa, Wd, Wd, Wa, Wa, Wd, Wd, Wa,
				],
				n_rooms_width: 2,
				n_rooms_height: 2,
			},
			RoomPrefab {
				chance: 3.0,
				tiles: vec![
					Wa, Wu, Ga, Wa,
					Wl, Fa, Fa, Wr,
					Ga, Fa, Fa, Ga,
					Wa, Fa, Fa, Wa,
					Wa, Fa, Fa, Wa,
					Wl, Fa, Fa, Wr,
					Ga, Fa, Fa, Ga,
					Wa, Wd, Ga, Wa,
				],
				n_rooms_width: 1,
				n_rooms_height: 2,
			},
			RoomPrefab {
				chance: 1.0,
				tiles: vec![
					Wa, Wu, Ga, Wa,
					Wl, Fa, Fa, Wr,
					Ga, Fa, Ga, Ga,
					Wa, Fa, Ga, Wa,
					Wa, Fa, Wr, Wr,
					Wl, Fa, Wr, Wr,
					Ga, Fa, Fa, Ga,
					Wa, Wd, Ga, Wa,
				],
				n_rooms_width: 1,
				n_rooms_height: 2,
			},
		];

		let total_prefab_chance: f32 = room_prefabs.iter().map(|v| v.chance).sum();

		let mut empty_spots = Vec::new();
		'main_generator: while !loose_ends.is_empty() {
			let loose_end_id = random.get_32() as usize % loose_ends.len();
			let loose_end = loose_ends.swap_remove(loose_end_id);

			let (off_x, off_y) = loose_end.direction.to_vec();

			let mut room_we_want = random.get_float() * total_prefab_chance;
			let mut wanted_prefab_id = 0;
			for (i, prefab) in room_prefabs.iter().enumerate() {
				room_we_want -= prefab.chance;

				if room_we_want <= 0.0 {
					wanted_prefab_id = i;
					break;
				}
			}
			let mut wanted_prefab = &room_prefabs[wanted_prefab_id];

			empty_spots.clear();
			rooms.find_empty_spot(
				loose_end.from_x, loose_end.from_y,
				wanted_prefab.n_rooms_width  as isize,
				wanted_prefab.n_rooms_height as isize,
				loose_end.direction,
				&mut empty_spots,
			);

			let (room_x, room_y) = match empty_spots.len() {
				0 => {
					if !rooms.square_is_empty(loose_end.from_x + off_x, loose_end.from_y + off_y, 1, 1) {
						if random.get_float() < 0.05 {
							if let Some(room) = rooms.get_mut(loose_end.from_x + off_x, loose_end.from_y + off_y) {
								*room.as_mut().unwrap()
									.get_dir_mut(loose_end.direction.inverted()) = true;
								*rooms.get_mut(loose_end.from_x, loose_end.from_y).unwrap()
									.as_mut().unwrap()
									.get_dir_mut(loose_end.direction) = true;
							}
						}
						continue 'main_generator;
					}

					wanted_prefab_id = 0;
					wanted_prefab = &room_prefabs[wanted_prefab_id];
					(loose_end.from_x + off_x, loose_end.from_y + off_y)
				}
				_ => {
					let index = random.get_32() as usize % empty_spots.len();
					empty_spots[index]
				}
			};

			for prefab_y in 0..wanted_prefab.n_rooms_height {
				for prefab_x in 0..wanted_prefab.n_rooms_width {
					let room = rooms.get_mut(
						room_x + prefab_x as isize,
						room_y + prefab_y as isize,
					).unwrap();

					*room = Some(Room::new(wanted_prefab_id, prefab_x, prefab_y));
				}
			}

			*rooms.get_mut(loose_end.from_x + off_x, loose_end.from_y + off_y).unwrap()
				.as_mut().unwrap()
				.get_dir_mut(loose_end.direction.inverted()) = true;
			*rooms.get_mut(loose_end.from_x, loose_end.from_y).unwrap()
				.as_mut().unwrap()
				.get_dir_mut(loose_end.direction) = true;

			for prefab_y in 0..wanted_prefab.n_rooms_height {
				loose_ends.push(LooseEnd {
					from_x: room_x,
					from_y: room_y + prefab_y as isize,
					direction: Direction::Left,
				});

				loose_ends.push(LooseEnd {
					from_x: room_x + wanted_prefab.n_rooms_width as isize - 1,
					from_y: room_y + prefab_y as isize,
					direction: Direction::Right,
				});
			}

			for prefab_x in 0..wanted_prefab.n_rooms_width {
				loose_ends.push(LooseEnd {
					from_x: room_x + prefab_x as isize,
					from_y: room_y,
					direction: Direction::Up,
				});

				loose_ends.push(LooseEnd {
					from_x: room_x + prefab_x as isize,
					from_y: room_y + wanted_prefab.n_rooms_height as isize - 1,
					direction: Direction::Down,
				});
			}
		}

		let mut world = World {
			tiles: TileMap {
				width: self.n_rooms_width * ROOM_WIDTH,
				height: self.n_rooms_height * ROOM_HEIGHT,
				tiles: vec![
					Tile::new(TileKind::Wall);
					self.n_rooms_width * self.n_rooms_height * ROOM_WIDTH * ROOM_HEIGHT
				],
			},
			sprites: HashMap::new(),
			sprite_id_counter: NonZeroU32::new(1).unwrap(),
			entities: HashMap::new(),
			entity_id_counter: NonZeroU32::new(1).unwrap(),
		};


		for (room_y, chunk) in rooms.rooms.chunks(self.n_rooms_width).enumerate() {
			for (room_x, room) in chunk.iter().enumerate() {
				let room = room.as_ref().expect("Didn't fill the entire dungeon!?");

				let room_data = &room_prefabs[room.room_id];

				for tile_y in 0..ROOM_HEIGHT {
					for tile_x in 0..ROOM_WIDTH {
						let gen_tile = &room_data.tiles[
							(room.room_y * ROOM_HEIGHT + tile_y) * ROOM_WIDTH * room_data.n_rooms_width +
							room.room_x * ROOM_WIDTH + tile_x
						];

						let mut gen_tile_kind = &gen_tile.kind;
						if let &Some((ref new_kind, change_if_dir)) = &gen_tile.change_if {
							if room.get_dir(change_if_dir) {
								gen_tile_kind = &new_kind;
							}
						}

						let kind = match gen_tile_kind {
							GenTileKind::Floor => {
								if random.get_float() <= 0.010 {
									let pos = Vec2::new(
										(room_x * ROOM_WIDTH + tile_x) as f32 + 0.5,
										(room_y * ROOM_HEIGHT + tile_y) as f32 + 0.5
									);
									let sprite = world.insert_sprite(Texture::Rick, pos, 1.0, 0.0);
									world.insert_entity(Entity {
										move_drag: 0.0,
										vel: Vec2::new(
											(random.get_float() - 0.5) * 25.0,
											(random.get_float() - 0.5) * 25.0,
										),
										.. Entity::new(pos, 0.3, Some(sprite))
									});
								}
								for _ in 0 .. random.get_32() % 60 {
									let pos = Vec2::new(
										(room_x * ROOM_WIDTH + tile_x) as f32 + random.get_float(),
										(room_y * ROOM_HEIGHT + tile_y) as f32 + random.get_float() 
									);
									world.insert_sprite(Texture::Rick, pos, random.get_float() * 0.1 + 0.1, 1.0);
								}
								TileKind::Floor
							},
							GenTileKind::Wall => TileKind::Wall,
							GenTileKind::Window => TileKind::Window,
						};

						world.tiles.tiles[
							(room_y * ROOM_HEIGHT + tile_y) * ROOM_WIDTH * self.n_rooms_width +
							(room_x * ROOM_WIDTH + tile_x)
						].kind = kind;
					}
				}
			}
		}

		let player_id = world.insert_entity(Entity::new(start, 0.3, None));

		(player_id, world)
	}
}

// TODO: Move this into its own thing.
#[derive(Clone, Copy, Debug)]
pub enum Direction {
	Up,
	Down,
	Left,
	Right,
}

impl Direction {
	fn inverted(self) -> Direction {
		match self {
			Direction::Up    => Direction::Down,
			Direction::Down  => Direction::Up,
			Direction::Left  => Direction::Right,
			Direction::Right => Direction::Left,
		}
	}

	fn to_vec(self) -> (isize, isize) {
		match self {
			Direction::Up    => (0, -1),
			Direction::Down  => (0,  1),
			Direction::Left  => (-1, 0),
			Direction::Right => (1,  0),
		}
	}
}

struct RoomPrefab {
	chance: f32,
	tiles: Vec<GenTile>,
	n_rooms_width: usize,
	n_rooms_height: usize,
}

#[derive(Clone)]
struct Room {
	room_id: usize,
	room_x: usize,
	room_y: usize,
	left: bool,
	right: bool,
	up: bool,
	down: bool,
}

impl Room {
	fn new(room_id: usize, room_x: usize, room_y: usize) -> Self {
		Room {
			room_id,
			room_x,
			room_y,
			left: false,
			right: false,
			up: false,
			down: false,
		}
	}

	fn get_dir(&self, direction: Direction) -> bool {
		match direction {
			Direction::Up    => self.up,
			Direction::Down  => self.down,
			Direction::Left  => self.left,
			Direction::Right => self.right,
		}
	}

	fn get_dir_mut(&mut self, direction: Direction) -> &mut bool {
		match direction {
			Direction::Up => &mut self.up,
			Direction::Down => &mut self.down,
			Direction::Left => &mut self.left,
			Direction::Right => &mut self.right,
		}
	}
}

struct Rooms {
	rooms: Vec<Option<Room>>,
	width: usize,
	height: usize,
}

impl Rooms {
	fn new(width: usize, height: usize) -> Self {
		Self {
			rooms: vec![None; width * height],
			width,
			height,
		}
	}

	#[inline]
	fn get(&self, x: isize, y: isize) -> Option<&Option<Room>> {
		if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
			Some(&self.rooms[y as usize * self.width + x as usize])
		} else {
			None
		}
	}

	#[inline]
	fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut Option<Room>> {
		if x >= 0 && (x as usize) < self.width && y >= 0 && (y as usize) < self.height {
			Some(&mut self.rooms[y as usize * self.width + x as usize])
		} else {
			None
		}
	}

	/// This function finds all empty rectangles with (width, height),
	/// starting from (x, y), into a given direction.
	fn find_empty_spot(
		&self,
		x: isize, y: isize,
		width: isize, height: isize,
		direction: Direction,
		results: &mut Vec<(isize, isize)>,
	) {
		let mut streak = 0;
		match direction {
			Direction::Right => {
				results.extend((-height + 1 .. height).filter_map(|off_y| {
					if self.square_is_empty(x + 1, y + off_y, width, 1) {
						streak += 1;

						if streak >= height {
							Some((x + 1, y + off_y - height + 1))
						} else {
							None
						}
					} else {
						streak = 0;
						None
					}
				}));
			}
			Direction::Left => {
				results.extend((-height + 1 .. height).filter_map(|off_y| {
					if self.square_is_empty(x - width, y + off_y, width, 1) {
						streak += 1;

						if streak >= height {
							Some((x - width, y + off_y - height + 1))
						} else {
							None
						}
					} else {
						streak = 0;
						None
					}
				}));
			}
			Direction::Up => {
				results.extend((-width + 1 .. width).filter_map(|off_x| {
					if self.square_is_empty(x + off_x, y - height, 1, height) {
						streak += 1;

						if streak >= width {
							Some((x + off_x - width + 1, y - height))
						} else {
							None
						}
					} else {
						streak = 0;
						None
					}
				}));
			}
			Direction::Down => {
				results.extend((-width + 1 .. width).filter_map(|off_x| {
					if self.square_is_empty(x + off_x, y + 1, 1, height) {
						streak += 1;

						if streak >= width {
							Some((x + off_x - width + 1, y + 1))
						} else {
							None
						}
					} else {
						streak = 0;
						None
					}
				}));
			}
		}
	}

	/// Returns true if the square is inside the bounds of the rooms, and if there are no
	/// rooms there already.
	fn square_is_empty(&self, x: isize, y: isize, width: isize, height: isize) -> bool {
		for y in y .. y + height {
			for x in x .. x + width {
				match self.get(x, y) {
					Some(None) => (),
					_ => return false,
				}
			}
		}

		true
	}
}
