use crate::{float_range, Vec2};
use image::{Pixel, RgbaImage};
use std::marker::PhantomData;

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
        Self {
            buffer,
            stride,
            height,
            _phantom: PhantomData,
        }
    }

    pub fn draw_uv_row(
        &mut self,
        image: &RgbaImage,
        mut uv_start: Vec2,
        mut uv_end: Vec2,
        pos_y_start: f32,
        pos_y_end: f32,
        dimming: f32,
    ) {
        if pos_y_start < 0.0 && pos_y_end < 0.0 {
            return;
        }

        let pos_y_start = if pos_y_start < 0.0 {
            uv_start =
                Vec2::lerp_unclamped(uv_start, uv_end, -pos_y_start / (pos_y_end - pos_y_start));
            0.0
        } else {
            pos_y_start
        };
        let pos_y_end = if pos_y_end > 1.0 {
            uv_end = Vec2::lerp_unclamped(
                uv_end,
                uv_start,
                (pos_y_end - 1.0) / (pos_y_end - pos_y_start),
            );
            1.0
        } else {
            pos_y_end
        };

        let size = (pos_y_end - pos_y_start) * self.height as f32;
        for index in (pos_y_start * self.height as f32) as usize
            ..((pos_y_end * self.height as f32) as usize).min(self.height - 1)
        {
            let factor = (index as f32 - pos_y_start * self.height as f32) / size;
            let mut lerped = Vec2::lerp_unclamped(uv_start, uv_end, factor);
            lerped.x = lerped.x - lerped.x.floor();
            lerped.y = lerped.y - lerped.y.floor();

            let pix = image.get_pixel(
                (lerped.x * image.width() as f32) as u32,
                (lerped.y * image.height() as f32) as u32,
            );
            let dimmed = dim_color(pix.channels4(), dimming);

            unsafe {
                *self.buffer.add(index * self.stride) =
                    dimmed.0 as u32 * 256 * 256 + dimmed.1 as u32 * 256 + dimmed.2 as u32;
            }
        }
    }

    /// Draws a cropped image.
    ///
    /// All the floating point coordinates are normalized between 0 and 1.
    /// Assumes that crop_y_start is less than crop_y_end,
    /// and that pos_y_start is less than pos_y_end.
    pub fn draw_partial_image(
        &mut self,
        image: &RgbaImage,
        image_x: u32,
        mut crop_y_start: f32,
        mut crop_y_end: f32,
        pos_y_start: f32,
        pos_y_end: f32,
        dimming: f32,
    ) {
        // assert!(pos_y_start < pos_y_end);
        // assert!(crop_y_start < crop_y_end);
        if pos_y_start < 0.0 && pos_y_end < 0.0 || pos_y_start > 1.0 && pos_y_end > 1.0 {
            return;
        }

        let pos_y_start = if pos_y_start < 0.0 {
            crop_y_start +=
                (-pos_y_start / (pos_y_end - pos_y_start)) * (crop_y_end - crop_y_start);
            0.0
        } else {
            pos_y_start
        };
        let pos_y_end = if pos_y_end > 1.0 {
            crop_y_end -=
                ((pos_y_end - 1.0) / (pos_y_end - pos_y_start)) * (crop_y_end - crop_y_start);
            1.0
        } else {
            pos_y_end
        };

        let mut pixel_iter = float_range::range(
            crop_y_start * image.height() as f32,
            crop_y_end * image.height() as f32,
        );
        let mut from_pixel = pixel_iter.next().unwrap();
        let d_pixel = ((pos_y_end - pos_y_start) * self.height as f32)
            / ((crop_y_end - crop_y_start) * image.height() as f32);
        let mut self_y = pos_y_start * self.height as f32;
        for to_pixel in pixel_iter {
            let pix = image.get_pixel(image_x, (from_pixel as u32).min(image.height() - 1));
            let self_y_end = self_y + d_pixel * (to_pixel - from_pixel);

            if pix[3] > 0 {
                let dimmed = dim_color(pix.channels4(), dimming);

                for buffer_index in self_y as usize..(self_y_end as usize).min(self.height - 1) {
                    unsafe {
                        *self.buffer.add(buffer_index * self.stride) =
                            dimmed.0 as u32 * 256 * 256 + dimmed.1 as u32 * 256 + dimmed.2 as u32;
                    }
                }
            }

            self_y = self_y_end;
            from_pixel = to_pixel;
        }
    }
}

#[inline]
fn dim_color(color: (u8, u8, u8, u8), dim_factor: f32) -> (u8, u8, u8, u8) {
    (
        (color.0 as f32 * dim_factor * dim_factor) as u8,
        (color.1 as f32 * dim_factor * dim_factor) as u8,
        (color.2 as f32 * dim_factor) as u8,
        0,
    )
}
