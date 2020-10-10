use super::SpriteId;
use crate::id::{Id, IdMap};
use crate::Vec2;
use std::collections::HashMap;

create_id!(EntityId);

pub struct Entities {
	pub entities: IdMap<EntityId, Entity>,
	pub transforms: HashMap<EntityId, Transform>,
}

impl Entities {
	pub fn new() -> Self {
		Self {
			entities: IdMap::new(),
			transforms: HashMap::new(),
		}
	}

	pub fn insert(&mut self, entity: Entity) -> EntityId {
		self.entities.insert(entity)
	}
}

#[derive(Clone, Copy, Default)]
pub struct Transform {
	pub pos: Vec2,
	pub vel: Vec2,
	pub drag: f32,
	pub rot: f32,
	pub size: f32,
	pub sprite: Option<SpriteId>,
}

#[derive(Default)]
pub struct Entity {
	pub can_open_doors: bool,
}
