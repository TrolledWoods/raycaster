pub trait SignedIndex<T> {
	type Output;

	fn iget(&self, index: T) -> Option<&Self::Output>;
	fn iget_mut(&mut self, index: T) -> Option<&mut Self::Output>;
}

impl<T> SignedIndex<isize> for [T] {
	type Output = T;

	fn iget(&self, index: isize) -> Option<&Self::Output> {
		if index >= 0 && (index as usize) < self.len() {
			// SAFETY: We did boundscheck above
			Some(unsafe { self.get_unchecked(index as usize) })
		} else {
			None
		}
	}

	fn iget_mut(&mut self, index: isize) -> Option<&mut Self::Output> {
		if index >= 0 && (index as usize) < self.len() {
			// SAFETY: We did boundscheck above
			Some(unsafe { self.get_unchecked_mut(index as usize) })
		} else {
			None
		}
	}
}
