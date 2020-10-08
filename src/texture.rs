macro_rules! create_textures {
	($($name:ident = $file_name:tt),*,) => {
		#[derive(Clone, Copy, PartialEq, Eq, Hash)]
		#[repr(u16)]
		pub enum Texture {
			$($name),*
		}

		const TEXTURE_FILES: &[&str] = &[
			$($file_name),*
		];
	}
}

create_textures!(
	Wall = "assets/wall.png",
	Window = "assets/window.png",
	Evil = "assets/evil.png",
	Rick = "assets/rick.png",
	Floor = "assets/floor.png",
	Fungus = "assets/fungus.png",
);

pub struct Textures {
	textures: Vec<image::RgbaImage>,
}

impl Textures {
	pub fn new() -> image::ImageResult<Self> {
		Ok(Textures {
			textures: TEXTURE_FILES
				.iter()
				.map(|&path| image::open(path).map(|v| v.into_rgba()))
				.collect::<Result<Vec<_>, image::ImageError>>()?,
		})
	}

	pub fn get(&self, texture: Texture) -> &image::RgbaImage {
		&self.textures[texture as u16 as usize]
	}
}
