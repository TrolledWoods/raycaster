use super::{Sprite, SpriteId};
use crate::texture::{Animation, Texture};
use crate::Vec2;

pub struct TileMap {
    tiles: Vec<Tile>,
    width: usize,
    height: usize,
}

impl TileMap {
    pub fn new(tiles: Vec<Tile>, width: usize, height: usize) -> Self {
        assert_eq!(tiles.len(), width * height);

        Self {
            tiles,
            width,
            height,
        }
    }

    pub fn to_image(&self, file: &str) {
        use image::{ImageBuffer, Pixel, Rgba};

        let mut image =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::new(self.width as u32, self.height as u32);

        for y in 0..self.width {
            for x in 0..self.height {
                let (r, g, b, a) = match self.get(x as isize, y as isize).unwrap().kind {
                    TileKind::Floor => (255, 200, 200, 255),
                    TileKind::Wall => (50, 50, 50, 255),
                    TileKind::Window => (50, 50, 75, 255),
                    TileKind::Door(_) => (100, 200, 200, 255),
                };

                let pixel = Pixel::from_channels(r, g, b, a);
                image.put_pixel(x as u32, y as u32, pixel);
            }
        }

        image.save(file).unwrap();
    }

    pub fn move_sprite(&mut self, sprite_id: SpriteId, sprite: &mut Sprite, new_pos: Vec2) {
        let size = sprite.size;

        let old_top = (sprite.pos.y - size / 2.0).floor() as isize;
        let old_bottom = (sprite.pos.y + size / 2.0).floor() as isize;
        let old_left = (sprite.pos.x - size / 2.0).floor() as isize;
        let old_right = (sprite.pos.x + size / 2.0).floor() as isize;

        let new_top = (new_pos.y - size / 2.0).floor() as isize;
        let new_bottom = (new_pos.y + size / 2.0).floor() as isize;
        let new_left = (new_pos.x - size / 2.0).floor() as isize;
        let new_right = (new_pos.x + size / 2.0).floor() as isize;

        sprite.pos = new_pos;

        if old_top == new_top
            && old_bottom == new_bottom
            && new_left == old_left
            && old_right == new_right
        {
            return;
        }

        for y in old_top..=old_bottom {
            for x in old_left..=old_right {
                if let Some(tile) = self.get_mut(x, y) {
                    if let Some(loc) = tile.sprites_inside.iter().position(|&v| v == sprite_id) {
                        tile.sprites_inside.swap_remove(loc);
                    }
                }
            }
        }

        for y in new_top..=new_bottom {
            for x in new_left..=new_right {
                if let Some(tile) = self.get_mut(x, y) {
                    tile.sprites_inside.push(sprite_id);
                }
            }
        }
    }

    pub fn square_is_colliding(&self, pos: Vec2, size: f32) -> bool {
        tiles_in_square(pos, size).any(|(x, y)| self.tile_is_colliding(x, y))
    }

    pub fn tile_is_colliding(&self, x: isize, y: isize) -> bool {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            self.tiles[y as usize * self.width + x as usize].is_solid()
        } else {
            false
        }
    }

    #[inline]
    pub fn get(&self, x: isize, y: isize) -> Option<&Tile> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(&self.tiles[y as usize * self.width + x as usize])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut(&mut self, x: isize, y: isize) -> Option<&mut Tile> {
        if x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height {
            Some(&mut self.tiles[y as usize * self.width + x as usize])
        } else {
            None
        }
    }

    #[inline]
    pub fn get_mut_usize(&mut self, x: usize, y: usize) -> Option<&mut Tile> {
        if x < self.width && y < self.height {
            Some(&mut self.tiles[y * self.width + x])
        } else {
            None
        }
    }
}

#[derive(Clone)]
pub struct TileGraphics {
    pub texture: Animation,
    pub is_transparent: bool,
}

#[derive(Clone)]
pub struct Tile {
    graphics: Option<TileGraphics>,
    kind: TileKind,
    pub sprites_inside: Vec<SpriteId>,
    pub floor_gfx: Texture,
}

#[derive(Clone)]
pub enum TileKind {
    Floor,
    Wall,
    Window,
    Door(bool),
}

impl Tile {
    pub fn new(kind: TileKind) -> Self {
        Self::new_with_time(kind, 0.0)
    }

    pub fn new_with_time(kind: TileKind, time: f32) -> Self {
        let mut tile = Tile {
            graphics: None,
            kind: TileKind::Floor,
            floor_gfx: Texture::Floor,
            sprites_inside: Vec::new(),
        };
        tile.set_kind_with_time(kind, time);
        tile
    }

    #[allow(unused)]
    pub fn kind(&self) -> &TileKind {
        &self.kind
    }

    pub fn set_kind(&mut self, kind: TileKind) {
        self.set_kind_with_time(kind, 0.0);
    }

    pub fn set_kind_with_time(&mut self, kind: TileKind, time: f32) {
        self.graphics = match kind {
            TileKind::Floor => None,
            TileKind::Wall => Some(TileGraphics {
                texture: Animation::new_loop_with_time(Texture::Wall, time),
                is_transparent: false,
            }),
            TileKind::Window => Some(TileGraphics {
                texture: Animation::new_loop_with_time(Texture::Window, time),
                is_transparent: true,
            }),
            TileKind::Door(true) => Some(TileGraphics {
                texture: Animation::new_clamp_with_time(Texture::Door, time),
                is_transparent: true,
            }),
            TileKind::Door(false) => Some(TileGraphics {
                texture: Animation::new_clamp_with_time(Texture::DoorClose, time),
                is_transparent: true,
            }),
        };
        self.kind = kind;
    }

    pub fn get_graphics(&self) -> &Option<TileGraphics> {
        &self.graphics
    }

    pub fn is_solid(&self) -> bool {
        match self.kind {
            TileKind::Floor => false,
            TileKind::Wall => true,
            TileKind::Window => true,
            TileKind::Door(_) => false, // !open,
        }
    }
}

pub fn tiles_in_square(pos: Vec2, half_size: f32) -> impl Iterator<Item = (isize, isize)> {
    let left = (pos.x - half_size) as isize;
    let right = (pos.x + half_size) as isize;
    let up = (pos.y - half_size) as isize;
    let down = (pos.y + half_size) as isize;
    (left..=right).flat_map(move |x| (up..=down).map(move |y| (x, y)))
}
