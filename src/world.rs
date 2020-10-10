mod entity;
pub mod generate;
mod tiles;

use crate::id::{Id, IdMap};
use crate::texture::*;
use crate::Vec2;
pub use entity::{Entities, Entity, EntityId, Transform};
pub use tiles::{Tile, TileKind, TileMap};

create_id!(SpriteId);

pub struct World {
	pub tiles: TileMap,
	pub entities: Entities,
	sprites: IdMap<SpriteId, Sprite>,
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

		let texture_x = sprite.pos.x;
		let texture_y = sprite.pos.y;
		let size = sprite.size;

		let id = self.sprites.insert(sprite);

		for y in
			(texture_y - size / 2.0).floor() as isize..=(texture_y + size / 2.0).floor() as isize
		{
			for x in (texture_x - size / 2.0).floor() as isize
				..=(texture_x + size / 2.0).floor() as isize
			{
				if let Some(tile) = self.tiles.get_mut(x, y) {
					tile.sprites_inside.push(id);
				}
			}
		}

		id
	}

	pub fn get_sprite(&self, id: SpriteId) -> Option<&Sprite> {
		self.sprites.get(id)
	}

	pub fn simulate_physics(&mut self, time_step: f32, _world_time: f32) {
		for transform in self.entities.transforms.values_mut() {
			transform.pos.x += transform.vel.x * time_step;
			if self
				.tiles
				.square_is_colliding(transform.pos, transform.size)
			{
				transform.pos.x -= transform.vel.x * time_step;
				transform.vel.x *= -1.0;
			}

			transform.pos.y += transform.vel.y * time_step;
			if self
				.tiles
				.square_is_colliding(transform.pos, transform.size)
			{
				transform.pos.y -= transform.vel.y * time_step;
				transform.vel.y *= -1.0;
			}

			transform.vel -= transform.vel * transform.drag * time_step;

			if let Some(sprite_id) = transform.sprite {
				self.tiles.move_sprite(
					sprite_id,
					self.sprites.get_mut(sprite_id).unwrap(),
					transform.pos,
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
