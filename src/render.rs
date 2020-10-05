use std::marker::PhantomData;
use image::RgbaImage;
use crate::float_range;

pub struct ImageColumn<'a> {
	// The buffer, as well as buffer.add(stride), buffer.add(stride * 2) e.t.c. until
	// buffer.add(stride * (height - 1)) should only be accessed by this struct for 'a,
	// and should be valid for 'a.
	buffer: *mut u32,
	stride: usize,
	height: usize,
	_phantom: PhantomData<&'a mut [u32]>,
}

impl<'a> ImageColumn<'a> {
	pub unsafe fn from_raw(buffer: *mut u32, stride: usize, height: usize) -> Self {
		Self { buffer, stride, height, _phantom: PhantomData }
	}

	/// Draws a cropped image.
	///
	/// All the floating point coordinates are normalized between 0 and 1.
	/// Assumes that crop_y_start is less than crop_y_end,
	/// and that pos_y_start is less than pos_y_end.
	pub fn draw_partial_image(
		&mut self, image: &RgbaImage, image_x: u32,
		mut crop_y_start: f32, mut crop_y_end: f32,
		pos_y_start: f32, pos_y_end: f32,
		dimming: f32,
	) {
		debug_assert!(pos_y_start < pos_y_end);
		debug_assert!(crop_y_start < crop_y_end);
		if pos_y_start < 0.0 && pos_y_end < 0.0 ||
			pos_y_start > 1.0 && pos_y_end > 1.0 {
			return;
		}

		let pos_y_start = if pos_y_start < 0.0 {
			crop_y_start +=
				(-pos_y_start / (pos_y_end - pos_y_start))
				* (crop_y_end - crop_y_start);
			0.0
		} else {
			pos_y_start
		};
		let pos_y_end = if pos_y_end > 1.0 {
			crop_y_end -=
				((pos_y_end - 1.0) / (pos_y_end - pos_y_start))
				* (crop_y_end - crop_y_start);
			1.0
		} else {
			pos_y_end
		};

		let mut pixel_iter = float_range::range(crop_y_start * image.height() as f32, crop_y_end * image.height() as f32);
		let mut from_pixel = pixel_iter.next().unwrap();
		let d_pixel = 
			((pos_y_end - pos_y_start) * self.height as f32)
			/ ((crop_y_end - crop_y_start) * image.height() as f32);
		let mut self_y = pos_y_start * self.height as f32;
		for to_pixel in pixel_iter {
			let pix = image.get_pixel(image_x, (from_pixel as u32).min(image.height() - 1));
			let self_y_end = self_y + d_pixel * (to_pixel - from_pixel);

			if pix[3] > 0 {
				let dimmed = u32::from_le_bytes(dim_color(pix.0, dimming));

				for buffer_index in self_y as usize .. (self_y_end as usize).min(self.height - 1) {
					unsafe {
						*self.buffer.add(buffer_index * self.stride) = dimmed;
					}
				}
			}

			self_y = self_y_end;
			from_pixel = to_pixel;
		}
	}
}

#[inline]
fn dim_color(color: [u8; 4], dim_factor: f32) -> [u8; 4] {
	[
		(color[0] as f32 * dim_factor) as u8,
		(color[1] as f32 * dim_factor) as u8,
		(color[2] as f32 * dim_factor) as u8,
		color[3],
	]
}