pub struct Textures {
	textures: Vec<image::RgbaImage>,
}

impl Textures {
	pub fn create(textures: Vec<image::RgbaImage>) -> Self {
		Textures {
			textures
		}
	}

	pub fn get(&self, index: u16) -> &image::RgbaImage {
		&self.textures[index as usize]
	}
}
