use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct MyAllocator {
	inner: System,
	pub allocations_count: AtomicUsize,
}

unsafe impl GlobalAlloc for MyAllocator {
	unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
		self.allocations_count.fetch_add(1, Ordering::Relaxed);
		self.inner.alloc(layout)
	}

	unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
		self.inner.dealloc(ptr, layout)
	}

	unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
		self.allocations_count.fetch_add(1, Ordering::Relaxed);
		self.inner.alloc_zeroed(layout)
	}

	unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
		self.allocations_count.fetch_add(1, Ordering::Relaxed);
		self.inner.realloc(ptr, layout, new_size)
	}
}

#[global_allocator]
pub static ALLOCATOR: MyAllocator = MyAllocator {
	inner: System,
	allocations_count: AtomicUsize::new(0),
};
