mod entity;
pub mod generate;
mod tiles;

use crate::texture::*;
use crate::Vec2;
pub use entity::{Entities, Entity, EntityId};
use std::collections::HashMap;
use std::num::NonZeroU32;
pub use tiles::{Tile, TileKind, TileMap};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteId(NonZeroU32);

pub struct World {
	pub tiles: TileMap,

	pub entities: Entities,

	sprites: HashMap<NonZeroU32, Sprite>,
	sprite_id_counter: NonZeroU32,
}

impl World {
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

	pub fn get_sprite(&self, id: SpriteId) -> Option<&Sprite> {
		self.sprites.get(&id.0)
	}

	pub fn simulate_physics(&mut self, time_step: f32, world_time: f32) {
		for entity in self.entities.iter_mut() {
			entity.pos.x += entity.vel.x * time_step;
			if self.tiles.square_is_colliding(entity.pos, entity.size) {
				if entity.can_open_doors {
					self.tiles.touch_event(entity, world_time);
				}
				entity.pos.x -= entity.vel.x * time_step;
				entity.vel.x *= -1.0;
			}

			entity.pos.y += entity.vel.y * time_step;
			if self.tiles.square_is_colliding(entity.pos, entity.size) {
				if entity.can_open_doors {
					self.tiles.touch_event(entity, world_time);
				}
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
