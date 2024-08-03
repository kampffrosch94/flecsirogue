use std::{
    collections::HashSet,
    ops::{Index, IndexMut, Not},
};

use crate::{grids::Grid, Player};
use ::rand::{rngs::StdRng, SeedableRng};
use flecs_ecs::prelude::*;
use itertools::Itertools;
use macroquad::prelude::*;
use mapgen::*;

use crate::Pos;

#[derive(Component)]
pub struct TileMap {
    pub w: i32,
    pub h: i32,
    pub terrain: Grid<TileKind>,
    pub visibility: Grid<Visibility>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileKind {
    Floor,
    Wall,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Unseen,
    Seen,
    Remembered,
}

impl TileMap {
    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(2234);
        let (w, h): (i32, i32) = (40, 40);
        let map = MapBuilder::new(w as _, h as _)
            .with(BspRooms::new())
            .with(NearestCorridors::new())
            .with(AreaStartingPosition::new(XStart::LEFT, YStart::BOTTOM))
            .with(DistantExit::new())
            .build_with_rng(&mut rng);
        let mut terrain = Grid::new(w, h, TileKind::Wall);
        for x in 0..w {
            for y in 0..h {
                if map.is_walkable(x as _, y as _) {
                    terrain[(x, y)] = TileKind::Floor;
                }
            }
        }
        let visibility = Grid::new(w, h, Visibility::Unseen);

        Self {
            w,
            h,
            terrain,
            visibility,
        }
    }
}

impl<T: Into<Pos>> Index<T> for TileMap {
    type Output = TileKind;

    fn index(&self, index: T) -> &Self::Output {
        let pos = index.into();
        &self.terrain[pos]
    }
}

impl<T: Into<Pos>> IndexMut<T> for TileMap {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let pos = index.into();
        &mut self.terrain[pos]
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
        world.set(TileMap::new());

        world
            .system_named::<&mut TileMap>("FOVClear")
            .term_at(0)
            .singleton()
            .each(|tm| {
                for v in tm.visibility.iter_values_mut() {
                    *v = match v {
                        Visibility::Seen => Visibility::Remembered,
                        _ => *v,
                    }
                }
            });
        world
            .system_named::<(&mut TileMap, &Pos)>("FOVRefresh")
            .term_at(0)
            .singleton()
            .with::<Player>()
            .each(|(tm, player_pos)| {
                for pos in floodfill_vision(tm, *player_pos) {
                    tm.visibility[pos] = Visibility::Seen;
                }
            });

        world
            .system_named::<(&TileMap, &WallSprite, &FloorSprite)>("DrawTilemap")
            .term_at(0)
            .singleton()
            .term_at(1)
            .singleton()
            .term_at(2)
            .singleton()
            .each(|(tm, wall_s, floor_s)| {
                for pos in tm.terrain.coords() {
                    let (fx, fy) = (pos.x as f32 * 32., pos.y as f32 * 32.);
                    let color = match tm.visibility[pos] {
                        Visibility::Unseen => BLACK,
                        Visibility::Seen => WHITE,
                        Visibility::Remembered => DARKGRAY,
                    };
                    match tm.terrain[pos] {
                        TileKind::Floor => {
                            let s = floor_s;
                            draw_texture_ex(&s.texture, fx, fy, color, s.params.clone());
                        }
                        TileKind::Wall => {
                            let s = wall_s;
                            draw_texture_ex(&s.texture, fx, fy, color, s.params.clone());
                        }
                    };
                }
            });
    }
}

pub fn floodfill_vision(tm: &TileMap, start: Pos) -> Vec<Pos> {
    const VISION_RANGE: i32 = 5;
    let mut working_set = vec![start];
    let mut visible = HashSet::new();
    visible.insert(start);

    while !working_set.is_empty() {
        for pos in working_set.drain(..).collect_vec() {
            for nb in pos.neighbors() {
                const WORKING_RANGE: i32 = VISION_RANGE - 1;
                match (nb.distance(start), tm[nb]) {
                    (0..=WORKING_RANGE, TileKind::Floor) => {
                        if visible.contains(&nb).not() {
                            working_set.push(nb);
                        }
                        visible.insert(nb);
                    }
                    (0..=WORKING_RANGE, TileKind::Wall) => {
                        visible.insert(nb);
                    }
                    (VISION_RANGE, _) => {
                        visible.insert(nb);
                    }
                    _ => panic!("should be unreachable"),
                }
            }
        }
    }
    visible.into_iter().collect()
}
