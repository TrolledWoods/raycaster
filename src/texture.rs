use std::path::Path;

macro_rules! create_textures {
	($($name:ident = $file_name:tt $total_anim_time:tt),*,) => {
		#[derive(Clone, Copy, PartialEq, Eq, Hash)]
		#[repr(u16)]
		pub enum Texture {
			$($name),*
		}

		impl Texture {
			pub fn total_animation_time(self) -> f32 {
				match self {
					$(Texture::$name => $total_anim_time),*
				}
			}
		}

		const TEXTURE_DATA: &[(f32, &str)] = &[
			$(($total_anim_time, $file_name)),*
		];
	}
}

create_textures!(
	Wall = "assets\\wall" 1.0,
	Window = "assets\\window" 1.0,
	// Evil = "assets\\evil" 1.0,
	Rick = "assets\\rick" 1.0,
	Floor = "assets\\floor" 1.0,
	Fungus = "assets\\fungus" 1.0,
	Door = "assets\\door" 1.0,
	DoorClose = "assets\\door_close" 1.0,
);

struct TextureInfo {
	id: usize,
	n_animation_frames: usize,
	fps: f32,
}

pub struct Textures {
	textures: Vec<TextureInfo>,
	images: Vec<image::RgbaImage>,
}

impl Textures {
	pub fn new() -> image::ImageResult<Self> {
		let mut images = Vec::new();
		let textures =
			TEXTURE_DATA
				.iter()
				.map(|&(total_time, path)| {
					let path: &Path = path.as_ref();
					match path.exists() {
						true => {
							let mut n_animation_frames = 0;
							let id = images.len();
							println!("loading folder {:?}", path);
							for n in 0.. {
								let mut path = path.to_path_buf();
								path.push(format!("{}.png", n));
								if !path.exists() {
									break;
								}
								images.push(image::open(&path)?.into_rgba());
								println!(" * {:?}", path);
								n_animation_frames += 1;
							}

							if n_animation_frames == 0 {
								todo!("Error while reading textures! There are no textures in animation");
							}

							Ok(TextureInfo {
								id,
								n_animation_frames,
								fps: n_animation_frames as f32 / total_time,
							})
						}
						false => {
							let mut path = path.to_path_buf();
							path.set_extension("png");
							let id = images.len();
							images.push(image::open(&path)?.into_rgba());
							println!("loaded image {:?}", path);
							Ok(TextureInfo {
								id,
								n_animation_frames: 1,
								fps: 1.0,
							})
						}
					}
				})
				.collect::<Result<Vec<_>, image::ImageError>>()?;

		Ok(Self { textures, images })
	}

	pub fn get(&self, texture: Texture) -> &image::RgbaImage {
		&self.images[self.textures[texture as u16 as usize].id]
	}

	pub fn get_anim(&self, animation: &Animation, time: f32) -> &image::RgbaImage {
		let texture = &self.textures[animation.texture as u16 as usize];
		let n_frames = (time - animation.start_time) * texture.fps * animation.speed;

		let frame = match animation.kind {
			AnimationKind::Looping => n_frames as usize % texture.n_animation_frames,
			AnimationKind::Clamped => (n_frames as usize).min(texture.n_animation_frames - 1),
		};

		&self.images[texture.id + frame]
	}
}

#[derive(Clone, Copy)]
pub enum AnimationKind {
	Looping,
	Clamped,
}

#[derive(Clone)]
pub struct Animation {
	pub texture: Texture,
	pub start_time: f32,
	pub speed: f32,
	pub kind: AnimationKind,
}

impl Animation {
	pub fn new_loop_with_time(texture: Texture, start_time: f32) -> Self {
		Animation {
			texture,
			start_time,
			speed: 1.0,
			kind: AnimationKind::Looping,
		}
	}

	pub fn new_clamp_with_time(texture: Texture, start_time: f32) -> Self {
		Animation {
			texture,
			speed: 1.0,
			start_time,
			kind: AnimationKind::Clamped,
		}
	}

	pub fn new_loop(texture: Texture) -> Self {
		Animation {
			texture,
			start_time: 0.0,
			speed: 1.0,
			kind: AnimationKind::Looping,
		}
	}

	pub fn new_clamp(texture: Texture) -> Self {
		Animation {
			texture,
			speed: 1.0,
			start_time: 0.0,
			kind: AnimationKind::Clamped,
		}
	}

	pub fn has_run_once(&self, time: f32) -> bool {
		time - self.start_time <= self.texture.total_animation_time() / self.speed
	}
}
