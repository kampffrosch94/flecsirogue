use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
};

use crate::{grids::Grid, Player, Sprite};
use ::rand::{rngs::StdRng, SeedableRng};
use flecs_ecs::prelude::*;
use macroquad::prelude::*;
use mapgen::*;

use crate::Pos;

#[derive(Component)]
pub struct TileMap {
    pub w: i32,
    pub h: i32,
    pub terrain: Grid<TileKind>,
    pub visibility: Grid<Visibility>,
    pub units: HashMap<Pos, Entity>,
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
            units: Default::default(),
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
    pub upper: Sprite,
    pub lower: Sprite,
}

#[derive(Component)]
pub struct FloorSprite {
    pub texture: Texture2D,
    pub params: DrawTextureParams,
}

#[derive(Component)]
pub struct Visible {}

#[derive(Component)]
pub struct TilemapModule {}

impl Module for TilemapModule {
    fn module(world: &flecs_ecs::prelude::World) {
        world.set(TileMap::new());

        world
            .system_named::<&mut TileMap>("TileMap:UnitClearPos")
            .term_at(0)
            .singleton()
            .each(|tm| {
                tm.units.clear();
            });

        world
            .system_named::<(&mut TileMap, &Pos)>("TileMap:UnitUpdatePos")
            .term_at(0)
            .singleton()
            .each_entity(|e, (tm, pos)| {
                tm.units.insert(*pos, *e);
            });

        world
            .system_named::<&mut TileMap>("TileMap:FOVClear")
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
            .system_named::<(&mut TileMap, &Pos)>("TileMap:FOVRefresh")
            .term_at(0)
            .singleton()
            .with::<Player>()
            .each(|(tm, player_pos)| {
                let terrain = &tm.terrain;
                let visibility = &mut tm.visibility;
                // TODO add max vision length by blocking everything from some range?
                let mut blocks_vision = |pos| terrain[Pos::from(pos)] == TileKind::Wall;
                let mut mark_visible = |pos| visibility[Pos::from(pos)] = Visibility::Seen;
                symmetric_shadowcasting::compute_fov(
                    player_pos.into(),
                    &mut blocks_vision,
                    &mut mark_visible,
                );
            });

        world
            .system_named::<(&mut TileMap, &Pos)>("TileMap:FOVMarkVisibleUnits")
            .term_at(0)
            .singleton()
            .with::<&mut Visible>()
            .optional()
            .each_entity(|e, (tm, pos)| {
                match tm.visibility[*pos] {
                    Visibility::Seen => e.add::<Visible>(),
                    _ => e.remove::<Visible>(),
                };
            });

        world
            .system_named::<(&TileMap, &WallSprite, &FloorSprite)>("TileMap:DrawTilemap")
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
                            let below = pos + (0, 1);
                            let s = match tm.terrain.get_opt(below) {
                                Some(TileKind::Wall) => &wall_s.upper,
                                _ => &wall_s.lower,
                            };
                            draw_texture_ex(&s.texture, fx, fy, color, s.params.clone());
                        }
                    };
                }
            });
    }
}
