pub mod generate;

use std::collections::HashMap;
use std::num::NonZeroU32;

use crate::texture::*;
use crate::Vec2;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(NonZeroU32);

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteId(NonZeroU32);

pub struct World {
	pub tiles: TileMap,

	entities: HashMap<NonZeroU32, Entity>,
	entity_id_counter: NonZeroU32,

	sprites: HashMap<NonZeroU32, Sprite>,
	sprite_id_counter: NonZeroU32,
}

impl World {
	pub fn insert_entity(&mut self, entity: Entity) -> EntityId {
		let id = self.entity_id_counter;
		self.entity_id_counter = NonZeroU32::new(self.entity_id_counter.get() + 1).unwrap();

		let old = self.entities.insert(id, entity);
		assert!(old.is_none());
		EntityId(id)
	}

	pub fn insert_sprite(
		&mut self,
		texture: Texture,
		pos: Vec2,
		size: f32,
		y_pos: f32,
	) -> SpriteId {
		let sprite = Sprite {
			texture,
			pos,
			size,
			y_pos,
		};

		let id = self.sprite_id_counter;
		self.sprite_id_counter = NonZeroU32::new(self.sprite_id_counter.get() + 1).unwrap();

		let texture_x = sprite.pos.x;
		let texture_y = sprite.pos.y;
		let size = sprite.size;

		let old = self.sprites.insert(id, sprite);
		assert!(old.is_none());

		for y in
			(texture_y - size / 2.0).floor() as isize..=(texture_y + size / 2.0).floor() as isize
		{
			for x in (texture_x - size / 2.0).floor() as isize
				..=(texture_x + size / 2.0).floor() as isize
			{
				if let Some(tile) = self.tiles.get_mut(x, y) {
					tile.sprites_inside.push(SpriteId(id));
				}
			}
		}

		SpriteId(id)
	}

	pub fn entities(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
		self.entities
			.iter()
			.map(|(&key, value)| (EntityId(key), value))
	}

	pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
		self.entities.get(&id.0)
	}

	pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
		self.entities.get_mut(&id.0)
	}

	pub fn get_sprite(&self, id: SpriteId) -> Option<&Sprite> {
		self.sprites.get(&id.0)
	}

	pub fn simulate_physics(&mut self, time_step: f32) {
		for (entity_id, entity) in self.entities.iter_mut() {
			entity.pos.x += entity.vel.x * time_step;
			if self.tiles.square_is_colliding(entity.pos, entity.size) {
				entity.pos.x -= entity.vel.x * time_step;
				entity.vel.x *= -1.0;
			}

			entity.pos.y += entity.vel.y * time_step;
			if self.tiles.square_is_colliding(entity.pos, entity.size) {
				entity.pos.y -= entity.vel.y * time_step;
				entity.vel.y *= -1.0;
			}

			entity.vel -= entity.vel * entity.move_drag * time_step;

			if let Some(sprite_id) = entity.sprite {
				self.tiles.move_sprite(
					sprite_id,
					self.sprites.get_mut(&sprite_id.0).unwrap(),
					entity.pos,
				);
			}
		}
	}

	pub fn to_image(&self, file: &str) {
		use image::{ImageBuffer, Pixel, Rgba};

		let mut image = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(
			self.tiles.width as u32,
			self.tiles.height as u32,
		);

		for y in 0..self.tiles.width {
			for x in 0..self.tiles.height {
				let (r, g, b, a) = match self.tiles.get(x as isize, y as isize).unwrap().kind {
					TileKind::Floor => (255, 200, 200, 255),
					TileKind::Wall => (50, 50, 50, 255),
					TileKind::Window => (50, 50, 75, 255),
					TileKind::Door(_) => (100, 200, 200, 255),
				};

				let pixel = Pixel::from_channels(r, g, b, a);
				image.put_pixel(x as u32, y as u32, pixel);
			}
		}

		image.save(file).unwrap();
	}
}

pub struct TileMap {
	tiles: Vec<Tile>,
	width: usize,
	height: usize,
}

impl TileMap {
	pub fn move_sprite(&mut self, sprite_id: SpriteId, sprite: &mut Sprite, new_pos: Vec2) {
		let size = sprite.size;

		let old_top = (sprite.pos.y - size / 2.0).floor() as isize;
		let old_bottom = (sprite.pos.y + size / 2.0).floor() as isize;
		let old_left = (sprite.pos.x - size / 2.0).floor() as isize;
		let old_right = (sprite.pos.x + size / 2.0).floor() as isize;

		let new_top = (new_pos.y - size / 2.0).floor() as isize;
		let new_bottom = (new_pos.y + size / 2.0).floor() as isize;
		let new_left = (new_pos.x - size / 2.0).floor() as isize;
		let new_right = (new_pos.x + size / 2.0).floor() as isize;

		sprite.pos = new_pos;

		if old_top == new_top
			&& old_bottom == new_bottom
			&& new_left == old_left
			&& old_right == new_right
		{
			return;
		}

		for y in old_top..=old_bottom {
			for x in old_left..=old_right {
				if let Some(tile) = self.get_mut(x, y) {
					if let Some(loc) = tile.sprites_inside.iter().position(|&v| v == sprite_id) {
						tile.sprites_inside.swap_remove(loc);
					}
				}
			}
		}

		for y in new_top..=new_bottom {
			for x in new_left..=new_right {
				if let Some(tile) = self.get_mut(x, y) {
					tile.sprites_inside.push(sprite_id);
				}
			}
		}
	}

	pub fn square_is_colliding(&self, pos: Vec2, size: f32) -> bool {
		tiles_in_square(pos, size).any(|(x, y)| self.tile_is_colliding(x, y))
	}

	pub fn tile_is_colliding(&self, x: isize, y: isize) -> bool {
		if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
			self.tiles[y as usize * self.width + x as usize].is_solid()
		} else {
			false
		}
	}

	#[inline]
	pub fn get(&self, x: isize, y: isize) -> Option<&Tile> {
		if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
			Some(&self.tiles[y as usize * self.width + x as usize])
		} else {
			None
		}
	}

	#[inline]
	pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut Tile> {
		if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
			Some(&mut self.tiles[y as usize * self.width + x as usize])
		} else {
			None
		}
	}
}

#[derive(Clone)]
pub struct TileGraphics {
	pub texture: Animation,
	pub is_transparent: bool,
}

#[derive(Clone)]
pub struct Tile {
	graphics: Option<TileGraphics>,
	kind: TileKind,
	pub sprites_inside: Vec<SpriteId>,
	pub floor_gfx: Texture,
}

#[derive(Clone)]
pub enum TileKind {
	Floor,
	Wall,
	Window,
	Door(bool),
}

impl Tile {
	pub fn new(kind: TileKind) -> Self {
		let mut tile = Tile {
			graphics: None,
			kind: TileKind::Floor,
			floor_gfx: Texture::Floor,
			sprites_inside: Vec::new(),
		};
		tile.set_kind(kind);
		tile
	}

	pub fn kind(&self) -> &TileKind {
		&self.kind
	}

	pub fn set_kind(&mut self, kind: TileKind) {
		self.graphics = match kind {
			TileKind::Floor => None,
			TileKind::Wall => Some(TileGraphics {
				texture: Animation::new_loop(Texture::Wall),
				is_transparent: false,
			}),
			TileKind::Window => Some(TileGraphics {
				texture: Animation::new_loop(Texture::Window),
				is_transparent: true,
			}),
			TileKind::Door(true) => Some(TileGraphics {
				texture: Animation::new_clamp(Texture::Door),
				is_transparent: true,
			}),
			TileKind::Door(false) => Some(TileGraphics {
				texture: Animation::new_clamp(Texture::Door),
				is_transparent: true,
			}),
		};
		self.kind = kind;
	}

	pub fn get_graphics(&self) -> &Option<TileGraphics> {
		&self.graphics
	}

	pub fn is_solid(&self) -> bool {
		match self.kind {
			TileKind::Floor => false,
			TileKind::Wall => true,
			TileKind::Window => true,
			TileKind::Door(open) => !open,
		}
	}
}

pub struct Sprite {
	pos: Vec2,
	pub y_pos: f32,
	pub texture: Texture,
	size: f32,
}

impl Sprite {
	#[inline]
	pub fn pos(&self) -> Vec2 {
		self.pos
	}

	#[inline]
	pub fn size(&self) -> f32 {
		self.size
	}
}

pub struct Entity {
	pub pos: Vec2,
	pub vel: Vec2,
	pub move_drag: f32,
	pub rot: f32,
	pub size: f32,
	pub sprite: Option<SpriteId>,
	pub can_open_doors: bool,
}

impl Entity {
	pub fn new(pos: Vec2, size: f32, sprite: Option<SpriteId>) -> Self {
		Entity {
			pos,
			vel: Vec2::zero(),
			move_drag: 1.0,
			rot: 0.0,
			size,
			sprite,
			can_open_doors: true,
		}
	}
}

pub fn tiles_in_square(pos: Vec2, half_size: f32) -> impl Iterator<Item = (isize, isize)> {
	let left = (pos.x - half_size) as isize;
	let right = (pos.x + half_size) as isize;
	let up = (pos.y - half_size) as isize;
	let down = (pos.y + half_size) as isize;
	(left..=right).flat_map(move |x| (up..=down).map(move |y| (x, y)))
}
