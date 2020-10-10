use std::collections::HashMap;
use std::hash::Hash;

pub trait Id {
	fn new() -> Self;
	fn create(value: u32) -> Self;
	fn get(&self) -> u32;
	fn count_next(&mut self) -> Self;
}

macro_rules! create_id {
	($(#[$meta_data:meta])* $name:ident) => {
		$(#[$meta_data])*
		#[derive(Clone, Copy, PartialEq, Eq, Hash)]
		pub struct $name(std::num::NonZeroU32);

		impl $name {
			#[allow(unused)]
			pub const fn create_raw(num: u32) -> Self {
				// TODO: Find a way to remove the unsafe here. The problem is that unwrap does not
				// work in constants, so we have to get it unchecked.
				Self(unsafe { std::num::NonZeroU32::new_unchecked(num + 1) })
			}

			#[allow(unused)]
			pub fn into_index(self) -> usize {
				(self.0.get() - 1) as usize
			}
		}

		impl Id for $name {
			#[allow(unused)]
			fn new() -> Self {
				Self(std::num::NonZeroU32::new(1).unwrap())
			}

			#[allow(unused)]
			fn get(&self) -> u32 {
				self.0.get() - 1
			}

			#[allow(unused)]
			fn create(value: u32) -> Self {
				Self(std::num::NonZeroU32::new(value + 1).unwrap())
			}

			#[allow(unused)]
			fn count_next(&mut self) -> Self {
				let value = *self;
				self.0 = std::num::NonZeroU32::new(self.0.get() + 1).unwrap();
				value
			}
		}

		impl std::fmt::Debug for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(f, stringify!($name))?;
				write!(f, "({})", self.into_index())?;
				Ok(())
			}
		}

		#[allow(unused_qualification)]
		impl std::fmt::Display for $name {
			fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
				write!(f, "{}", self.into_index())
			}
		}
	}
}

#[derive(Debug)]
pub struct IdVec<T, I>
where
	I: Id,
{
	contents: Vec<T>,
	_phantom: std::marker::PhantomData<I>,
}

impl<T, I> Clone for IdVec<T, I>
where
	T: Clone,
	I: Id,
{
	fn clone(&self) -> Self {
		Self {
			contents: self.contents.clone(),
			_phantom: std::marker::PhantomData,
		}
	}
}

// impl<T, I> Default for IdVec<T, I>
// where
// 	I: Id,
// {
// 	fn default() -> Self {
// 		Self {
// 			contents: Vec::new(),
// 			_phantom: std::marker::PhantomData,
// 		}
// 	}
// }
//
// impl<T, I> IdVec<T, I>
// where
// 	I: Id,
// {
// 	pub fn new() -> Self {
// 		Self {
// 			contents: Vec::new(),
// 			_phantom: std::marker::PhantomData,
// 		}
// 	}
//
// 	pub fn iter_ids(&self) -> impl Iterator<Item = (I, &T)> {
// 		self.contents
// 			.iter()
// 			.enumerate()
// 			.map(|(i, v)| (I::create(i as u32), v))
// 	}
//
// 	pub fn get(&self, index: I) -> &T {
// 		&self.contents[index.get() as usize]
// 	}
//
// 	pub fn get_mut(&mut self, index: I) -> &mut T {
// 		&mut self.contents[index.get() as usize]
// 	}
//
// 	pub fn push(&mut self, item: T) -> I {
// 		let id = self.contents.len() as u32;
// 		self.contents.push(item);
// 		I::create(id)
// 	}
// }

impl<T, I> std::ops::Deref for IdVec<T, I>
where
	I: Id,
{
	type Target = [T];

	fn deref(&self) -> &Self::Target {
		&*self.contents
	}
}

impl<T, I> std::ops::DerefMut for IdVec<T, I>
where
	I: Id,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut *self.contents
	}
}

pub struct IdMap<K, V> {
	internal: HashMap<K, V>,
	id_counter: u32,
}

impl<K, V> Default for IdMap<K, V> {
	fn default() -> Self {
		Self {
			internal: HashMap::new(),
			id_counter: 0,
		}
	}
}

impl<K, V> IdMap<K, V>
where
	K: Id + Eq + Hash + Copy,
{
	pub fn new() -> Self {
		Self {
			internal: HashMap::new(),
			id_counter: 0,
		}
	}

	pub fn insert(&mut self, value: V) -> K {
		let id = K::create(self.id_counter);
		self.id_counter += 1;
		let _ = self.internal.insert(id, value);
		id
	}

	#[allow(unused)]
	pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
		self.internal.values_mut()
	}

	pub fn get(&self, id: K) -> Option<&V> {
		self.internal.get(&id)
	}

	pub fn get_mut(&mut self, id: K) -> Option<&mut V> {
		self.internal.get_mut(&id)
	}
}
