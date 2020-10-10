use super::{entity, Entities, Entity, EntityId, Tile, TileKind, TileMap, Transform, World};
use crate::id::IdMap;
use crate::random::Random;
use crate::texture::Texture;
use crate::Vec2;

const ROOM_WIDTH: usize = 4;
const ROOM_HEIGHT: usize = 4;

#[derive(Clone)]
pub enum GenTileKind {
	Floor,
	Wall,
	Window,
	Door,
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

	pub fn change_if(mut self, kind: GenTileKind, dir: Direction) -> Self {
		self.change_if = Some((kind, dir));
		self
	}
}

pub struct WorldGenerator {
	n_rooms_width: usize,
	n_rooms_height: usize,
	prefabs: Vec<RoomPrefab>,
}

impl WorldGenerator {
	pub fn new(
		n_rooms_width: usize,
		n_rooms_height: usize,
		prefabs_path: &str,
	) -> Result<Self, &'static str> {
		Ok(Self {
			n_rooms_width,
			n_rooms_height,
			prefabs: load_prefabs_from_path(prefabs_path)?,
		})
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
		*rooms
			.get_mut(
				(start.x.floor() / ROOM_WIDTH as f32) as isize,
				(start.y.floor() / ROOM_HEIGHT as f32) as isize,
			)
			.unwrap() = Some(Room::new(0, 0, 0));

		let total_prefab_chance: f32 = self.prefabs.iter().map(|v| v.chance).sum();

		let mut entities = Vec::new();
		let mut empty_spots = Vec::new();
		'main_generator: while !loose_ends.is_empty() {
			let loose_end_id = random.get_32() as usize % loose_ends.len();
			let loose_end = loose_ends.swap_remove(loose_end_id);

			let (off_x, off_y) = loose_end.direction.to_vec();

			let mut room_we_want = random.get_float() * total_prefab_chance;
			let mut wanted_prefab_id = 0;
			for (i, prefab) in self.prefabs.iter().enumerate() {
				room_we_want -= prefab.chance;

				if room_we_want <= 0.0 {
					wanted_prefab_id = i;
					break;
				}
			}
			let mut wanted_prefab = &self.prefabs[wanted_prefab_id];

			empty_spots.clear();
			rooms.find_empty_spot(
				loose_end.from_x,
				loose_end.from_y,
				wanted_prefab.n_rooms_width as isize,
				wanted_prefab.n_rooms_height as isize,
				loose_end.direction,
				&mut empty_spots,
			);

			let (room_x, room_y) = match empty_spots.len() {
				0 => {
					if !rooms.square_is_empty(
						loose_end.from_x + off_x,
						loose_end.from_y + off_y,
						1,
						1,
					) {
						if random.get_float() < 0.05 {
							if let Some(room) =
								rooms.get_mut(loose_end.from_x + off_x, loose_end.from_y + off_y)
							{
								*room
									.as_mut()
									.unwrap()
									.get_dir_mut(loose_end.direction.inverted()) = true;
								*rooms
									.get_mut(loose_end.from_x, loose_end.from_y)
									.unwrap()
									.as_mut()
									.unwrap()
									.get_dir_mut(loose_end.direction) = true;
							}
						}
						continue 'main_generator;
					}

					wanted_prefab_id = 0;
					wanted_prefab = &self.prefabs[wanted_prefab_id];
					(loose_end.from_x + off_x, loose_end.from_y + off_y)
				}
				_ => {
					let index = random.get_32() as usize % empty_spots.len();
					empty_spots[index]
				}
			};

			for prefab_y in 0..wanted_prefab.n_rooms_height {
				for prefab_x in 0..wanted_prefab.n_rooms_width {
					let room = rooms
						.get_mut(room_x + prefab_x as isize, room_y + prefab_y as isize)
						.unwrap();

					*room = Some(Room::new(wanted_prefab_id, prefab_x, prefab_y));
				}
			}

			for &(pos, ref entity_kind) in wanted_prefab.entities.iter() {
				entities.push((
					pos + Vec2::new(
						room_x as f32 * ROOM_WIDTH as f32,
						room_y as f32 * ROOM_HEIGHT as f32,
					),
					entity_kind.clone(),
				));
			}

			*rooms
				.get_mut(loose_end.from_x + off_x, loose_end.from_y + off_y)
				.unwrap()
				.as_mut()
				.unwrap()
				.get_dir_mut(loose_end.direction.inverted()) = true;
			*rooms
				.get_mut(loose_end.from_x, loose_end.from_y)
				.unwrap()
				.as_mut()
				.unwrap()
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
			tiles: TileMap::new(
				vec![
					Tile::new(TileKind::Floor);
					self.n_rooms_width * self.n_rooms_height * ROOM_WIDTH * ROOM_HEIGHT
				],
				self.n_rooms_width * ROOM_WIDTH,
				self.n_rooms_height * ROOM_HEIGHT,
			),
			random: Random::new(),
			sprites: IdMap::new(),
			entities: Entities::new(),
		};

		eprintln!("Generating {} entities", entities.len());
		for (pos, entity) in entities {
			match entity {
				GenEntity::Evil => {
					let id = world.entities.insert(Default::default());
					let sprite_id = world.insert_sprite(Texture::Evil, pos, 0.4, 0.5);
					world.entities.transforms.insert(
						id,
						Transform {
							pos,
							drag: 0.4,
							size: 0.2,
							sprite: Some(sprite_id),
							..Default::default()
						},
					);
					world
						.entities
						.evil_enemies
						.insert(id, entity::EvilEnemy::Wander(0.0));
				}
			}
		}

		for (room_y, chunk) in rooms.rooms.chunks(self.n_rooms_width).enumerate() {
			for (room_x, room) in chunk.iter().enumerate() {
				let room = room.as_ref().expect("Didn't fill the entire dungeon!?");

				let room_data = &self.prefabs[room.room_id];

				for tile_y in 0..ROOM_HEIGHT {
					for tile_x in 0..ROOM_WIDTH {
						let gen_tile = &room_data.tiles[(room.room_y * ROOM_HEIGHT + tile_y)
							* ROOM_WIDTH * room_data.n_rooms_width
							+ room.room_x * ROOM_WIDTH + tile_x];

						let mut gen_tile_kind = &gen_tile.kind;
						if let &Some((ref new_kind, change_if_dir)) = &gen_tile.change_if {
							if room.get_dir(change_if_dir) {
								gen_tile_kind = &new_kind;
							}
						}

						let kind = match gen_tile_kind {
							GenTileKind::Door => TileKind::Door(false),
							GenTileKind::Floor => {
								if random.get_float() <= 0.001 {
									let pos = Vec2::new(
										(room_x * ROOM_WIDTH + tile_x) as f32 + 0.5,
										(room_y * ROOM_HEIGHT + tile_y) as f32 + 0.5,
									);
									let sprite = world.insert_sprite(Texture::Rick, pos, 1.0, 0.0);
									let entity_id = world.entities.insert(Entity {
										can_open_doors: false,
									});
									world.entities.transforms.insert(
										entity_id,
										Transform {
											drag: 0.0,
											vel: Vec2::new(
												(random.get_float() - 0.5) * 25.0,
												(random.get_float() - 0.5) * 25.0,
											),
											pos,
											size: 0.3,
											sprite: Some(sprite),
											..Default::default()
										},
									);
								}
								for _ in 0..(random.get_float()
									* random.get_float() * random.get_float()
									* 60.0) as u32
								{
									let pos = Vec2::new(
										(room_x * ROOM_WIDTH + tile_x) as f32 + random.get_float(),
										(room_y * ROOM_HEIGHT + tile_y) as f32 + random.get_float(),
									);
									world.insert_sprite(
										Texture::Fungus,
										pos,
										random.get_float() * 0.1 + 0.1,
										1.0,
									);
								}
								TileKind::Floor
							}
							GenTileKind::Wall => TileKind::Wall,
							GenTileKind::Window => TileKind::Window,
						};

						let tile = world
							.tiles
							.get_mut_usize(
								room_x * ROOM_WIDTH + tile_x,
								room_y * ROOM_HEIGHT + tile_y,
							)
							.unwrap();
						tile.set_kind(kind);
					}
				}
			}
		}

		let player_id = world.entities.insert(Entity {
			can_open_doors: true,
			..Default::default()
		});
		world.entities.transforms.insert(
			player_id,
			Transform {
				pos: start,
				size: 0.2,
				drag: 1.0,
				..Default::default()
			},
		);

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
			Direction::Up => Direction::Down,
			Direction::Down => Direction::Up,
			Direction::Left => Direction::Right,
			Direction::Right => Direction::Left,
		}
	}

	fn to_vec(self) -> (isize, isize) {
		match self {
			Direction::Up => (0, -1),
			Direction::Down => (0, 1),
			Direction::Left => (-1, 0),
			Direction::Right => (1, 0),
		}
	}
}

#[derive(Clone)]
enum GenEntity {
	Evil,
}

struct RoomPrefab {
	entities: Vec<(Vec2, GenEntity)>,
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
			Direction::Up => self.up,
			Direction::Down => self.down,
			Direction::Left => self.left,
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
		x: isize,
		y: isize,
		width: isize,
		height: isize,
		direction: Direction,
		results: &mut Vec<(isize, isize)>,
	) {
		let mut streak = 0;
		match direction {
			Direction::Right => {
				results.extend((-height + 1..height).filter_map(|off_y| {
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
				results.extend((-height + 1..height).filter_map(|off_y| {
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
				results.extend((-width + 1..width).filter_map(|off_x| {
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
				results.extend((-width + 1..width).filter_map(|off_x| {
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
		for y in y..y + height {
			for x in x..x + width {
				match self.get(x, y) {
					Some(None) => (),
					_ => return false,
				}
			}
		}

		true
	}
}

fn load_prefabs_from_path(path: &str) -> Result<Vec<RoomPrefab>, &'static str> {
	fn validate_room_prefab(prefab: &RoomPrefab) -> Result<(), &'static str> {
		if prefab.n_rooms_width == 0 {
			return Err("Prefab width cannot be zero");
		}
		if prefab.n_rooms_height == 0 {
			return Err("Prefab height cannot be zero");
		}
		if prefab.tiles.len()
			!= prefab.n_rooms_width * prefab.n_rooms_height * ROOM_WIDTH * ROOM_HEIGHT
		{
			return Err("Prefab tiles do not match up with with and height of room(make sure that width and height are divisible by ROOM_WIDTH and ROOM_HEIGHT");
		}

		Ok(())
	}

	let file_contents = std::fs::read_to_string(path).map_err(|_| "Couldn't load file")?;

	let mut prefabs = Vec::new();
	let mut current_prefab: Option<RoomPrefab> = None;
	let mut y = 0;

	for line in file_contents
		.lines()
		.map(|line| line.trim())
		.filter(|line| !line.is_empty())
	{
		let mut parts = line.split_whitespace();
		match parts.next().unwrap() {
			"--" => {
				if let Some(mut prefab) = current_prefab.take() {
					prefab.n_rooms_height = prefab
						.tiles
						.len()
						.checked_div(prefab.n_rooms_width * ROOM_WIDTH * ROOM_HEIGHT)
						.unwrap_or(0);

					validate_room_prefab(&prefab)?;
					prefabs.push(prefab);
				}

				let _name = parts.next().ok_or("Expected name of area");
				current_prefab = Some(RoomPrefab {
					entities: Vec::new(),
					chance: 0.0,
					tiles: Vec::new(),
					n_rooms_width: 0,
					n_rooms_height: 0,
				});
				y = 0;
			}
			"-" => {
				let prefab = current_prefab
					.as_mut()
					.ok_or("Can't set a property without an active room")?;
				match parts.next().ok_or("Expected property name")? {
					"chance" => {
						prefab.chance = parts
							.next()
							.ok_or("Expected float after 'chance'")?
							.parse::<f32>()
							.map_err(|_| "Float after 'chance' is incorrectly formatted")?;
					}
					_ => return Err("Unknown property"),
				}
			}
			body => {
				let prefab = current_prefab
					.as_mut()
					.ok_or("Can't set tiles without an active room")?;

				let mut body_chars = body.chars();
				let mut width = 0;
				let mut x = 0;
				while let Some(c) = body_chars.next() {
					let modifier = body_chars.next().ok_or("Expected tile modifier")?;
					prefab.tiles.push(match (c, modifier) {
						('#', '#') => GenTile::new(GenTileKind::Wall),
						('#', '>') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Floor, Direction::Right),
						('#', '<') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Floor, Direction::Left),
						('#', '^') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Floor, Direction::Up),
						('#', 'v') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Floor, Direction::Down),
						('o', 'o') => GenTile::new(GenTileKind::Window),
						('.', '.') => GenTile::new(GenTileKind::Floor),
						('E', '.') => {
							prefab
								.entities
								.push((Vec2::new(x as f32 + 0.5, y as f32 + 0.5), GenEntity::Evil));
							GenTile::new(GenTileKind::Floor)
						}
						('D', 'D') => GenTile::new(GenTileKind::Door),
						('D', '>') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Door, Direction::Right),
						('D', '<') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Door, Direction::Left),
						('D', '^') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Door, Direction::Up),
						('D', 'v') => GenTile::new(GenTileKind::Wall)
							.change_if(GenTileKind::Door, Direction::Down),
						_ => return Err("Invalid tile character"),
					});
					width += 1;
					x += 1;
				}

				if prefab.n_rooms_width == 0 {
					prefab.n_rooms_width = width / ROOM_WIDTH;
				} else if prefab.n_rooms_width * ROOM_WIDTH != width {
					return Err("Widths don't match");
				}

				y += 1;
			}
		}
	}

	if let Some(mut prefab) = current_prefab.take() {
		prefab.n_rooms_height = prefab
			.tiles
			.len()
			.checked_div(prefab.n_rooms_width * ROOM_WIDTH * ROOM_HEIGHT)
			.unwrap_or(0);

		validate_room_prefab(&prefab)?;
		prefabs.push(prefab);
	}

	Ok(prefabs)
}
