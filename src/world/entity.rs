use super::SpriteId;
use crate::id::{Id, IdMap};
use crate::Vec2;

create_id!(EntityId);

pub struct Entities {
	entities: IdMap<EntityId, Entity>,
}

impl Entities {
	pub fn new() -> Self {
		Self {
			entities: IdMap::new(),
		}
	}

	pub fn insert(&mut self, entity: Entity) -> EntityId {
		self.entities.insert(entity)
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
		self.entities.values_mut()
	}

	pub fn get(&self, id: EntityId) -> Option<&Entity> {
		self.entities.get(id)
	}

	pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
		self.entities.get_mut(id)
	}
}

#[derive(Clone, Copy)]
pub struct Transform {
	pub pos: Vec2,
	pub vel: Vec2,
	pub drag: f32,
	pub rot: f32,
	pub size: f32,
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
