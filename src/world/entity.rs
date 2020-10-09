use super::SpriteId;
use crate::Vec2;
use std::collections::HashMap;
use std::num::NonZeroU32;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct EntityId(NonZeroU32);

pub struct Entities {
	entities: HashMap<NonZeroU32, Entity>,
	entity_id_counter: NonZeroU32,
}

impl Entities {
	pub fn new() -> Self {
		Self {
			entities: HashMap::new(),
			entity_id_counter: NonZeroU32::new(1).unwrap(),
		}
	}

	pub fn insert(&mut self, entity: Entity) -> EntityId {
		let id = self.entity_id_counter;
		self.entity_id_counter = NonZeroU32::new(self.entity_id_counter.get() + 1).unwrap();

		let old = self.entities.insert(id, entity);
		assert!(old.is_none());
		EntityId(id)
	}

	pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> {
		self.entities.values_mut()
	}

	pub fn get(&self, id: EntityId) -> Option<&Entity> {
		self.entities.get(&id.0)
	}

	pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
		self.entities.get_mut(&id.0)
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
