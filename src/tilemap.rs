use std::ops::{Index, IndexMut};

use ::rand::{rngs::StdRng, SeedableRng};
use flecs_ecs::prelude::*;
use crate::grids;
use macroquad::prelude::*;
use mapgen::*;

use crate::Pos;

#[derive(Component)]
pub struct Tilemap {
    pub w: i32,
    pub h: i32,
    pub terrain: Vec<TileKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Floor,
    Wall,
}

impl Tilemap {
    fn index(&self, pos: Pos) -> usize {
        return (self.w * pos.y + pos.x) as usize;
    }

    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(2234);
        let (w, h) = (40, 40);
        let map = MapBuilder::new(w, h)
            .with(BspRooms::new())
            .with(NearestCorridors::new())
            .with(AreaStartingPosition::new(XStart::LEFT, YStart::BOTTOM))
            .with(DistantExit::new())
            .build_with_rng(&mut rng);
        let mut tiles = Vec::new();
        for x in 0..w {
            for y in 0..h {
                if map.is_walkable(x as _, y as _) {
                    tiles.push(TileKind::Floor);
                } else {
                    tiles.push(TileKind::Wall)
                }
            }
        }
        Self {
            w: w as _,
            h: h as _,
            terrain: tiles,
        }
    }
}

impl<T: Into<Pos>> Index<T> for Tilemap {
    type Output = TileKind;

    fn index(&self, index: T) -> &Self::Output {
        let pos = index.into();
        let index = self.index(pos);
        &self.terrain[index]
    }
}

impl<T: Into<Pos>> IndexMut<T> for Tilemap {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let pos = index.into();
        let index = self.index(pos);
        &mut self.terrain[index]
    }
}

#[derive(Component)]
pub struct WallSprite {
    pub texture: Texture2D,
    pub params: DrawTextureParams,
}

#[derive(Component)]
pub struct FloorSprite {
    pub texture: Texture2D,
    pub params: DrawTextureParams,
}

#[derive(Component)]
pub struct TilemapModule {}

impl Module for TilemapModule {
    fn module(world: &flecs_ecs::prelude::World) {
        world.set(Tilemap::new());
        world
            .system_named::<(&Tilemap, &WallSprite, &FloorSprite)>("DrawTilemap")
            .term_at(0)
            .singleton()
            .term_at(1)
            .singleton()
            .term_at(2)
            .singleton()
            .each(|(tm, wall_s, floor_s)| {
                for x in 0..tm.w {
                    for y in 0..tm.h {
                        let (fx, fy) = (x as f32 * 32., y as f32 * 32.);
                        match tm[(x, y)] {
                            TileKind::Floor => {
                                let s = floor_s;
                                draw_texture_ex(&s.texture, fx, fy, WHITE, s.params.clone());
                            }
                            TileKind::Wall => {
                                let s = wall_s;
                                draw_texture_ex(&s.texture, fx, fy, WHITE, s.params.clone());
                            }
                        };
                    }
                }
            });
    }
}
