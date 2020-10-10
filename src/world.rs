mod entity;
pub mod generate;
mod tiles;

use crate::id::{Id, IdMap};
use crate::random::Random;
use crate::texture::*;
use crate::Vec2;
pub use entity::{Entities, Entity, EntityId, Transform};
pub use tiles::{Tile, TileKind, TileMap};

create_id!(SpriteId);

pub struct World {
	pub tiles: TileMap,
	pub entities: Entities,
	sprites: IdMap<SpriteId, Sprite>,
	random: Random,
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

	pub fn simulate(&mut self, time_step: f32, _world_time: f32) {
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

		for (entity_id, evil_enemy) in self.entities.evil_enemies.iter_mut() {
			match evil_enemy {
				entity::EvilEnemy::Wander(time) => {
					*time -= time_step;

					if *time < 0.0 {
						self.entities
							.transforms
							.get_mut(&entity_id)
							.expect("Evil enemy needs a transform")
							.vel += Vec2::new(self.random.get_float() - 0.5, self.random.get_float() - 0.5);
						*time = self.random.get_float() * 3.0 + 1.0;
					}
				}
				entity::EvilEnemy::Angry(target) => match self.entities.transforms.get(target) {
					Some(target_transform) => {
						let target_pos = target_transform.pos;
						let evil_enemy_transform = self
							.entities
							.transforms
							.get_mut(&entity_id)
							.expect("Evil enemy needs a transform");

						evil_enemy_transform.vel +=
							(target_pos - evil_enemy_transform.pos) * time_step * 0.1;
					}
					None => {
						*evil_enemy = entity::EvilEnemy::Wander(2.0);
					}
				},
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
