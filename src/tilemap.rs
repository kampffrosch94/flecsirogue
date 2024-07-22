use std::ops::{Index, IndexMut};

use ::rand::{rngs::StdRng, SeedableRng};
use derive_more::From;
use flecs_ecs::prelude::*;
use macroquad::{prelude::*, texture::draw_texture};
use mapgen::*;

#[derive(Component)]
pub struct Tilemap {
    pub w: i32,
    pub h: i32,
    tiles: Vec<Tile>,
}

pub enum Tile {
    Floor,
    Wall,
}

/// TileMap coordinate
#[derive(Clone, Copy, Hash, PartialEq, From)]
struct Coord {
    x: i32,
    y: i32,
}

impl Tilemap {
    fn index(&self, pos: Coord) -> usize {
        return (self.w * pos.y + pos.x) as usize;
    }

    pub fn new() -> Self {
        let mut rng = StdRng::seed_from_u64(1234);
        let (w, h) = (80, 50);
        let map = MapBuilder::new(w, h)
            .with(NoiseGenerator::uniform())
            .with(CellularAutomata::new())
            .with(AreaStartingPosition::new(XStart::CENTER, YStart::CENTER))
            .with(CullUnreachable::new())
            .with(DistantExit::new())
            .build_with_rng(&mut rng);
        let mut tiles = Vec::new();
        for x in 0..w {
            for y in 0..h {
                if map.is_walkable(x as _, y as _) {
                    tiles.push(Tile::Floor);
                } else {
                    tiles.push(Tile::Wall)
                }
            }
        }
        Self {
            w: w as _,
            h: h as _,
            tiles,
        }
    }
}

impl<T: Into<Coord>> Index<T> for Tilemap {
    type Output = Tile;

    fn index(&self, index: T) -> &Self::Output {
        let pos = index.into();
        let index = self.index(pos);
        &self.tiles[index]
    }
}

impl<T: Into<Coord>> IndexMut<T> for Tilemap {
    fn index_mut(&mut self, index: T) -> &mut Self::Output {
        let pos = index.into();
        let index = self.index(pos);
        &mut self.tiles[index]
    }
}

/*
pub fn draw_tile_map(game: &Game) {
    let floor = game.textures["floor"];
    let wall = game.textures["wall"];
    for x in 0..game.map.w {
        for y in 0..game.map.h {
            let (fx, fy) = (x as f32 * 32., y as f32 * 32.);
            match game.map[(x, y)] {
                Tile::Floor => {
                    draw_texture(floor, fx, fy, WHITE);
                }
                Tile::Wall => {
                    draw_texture(wall, fx, fy, WHITE);
                }
            }
        }
    }
}

*/

#[derive(Component)]
pub struct TilemapModule {}

impl Module for TilemapModule {
    fn module(world: &flecs_ecs::prelude::World) {
        world.set(Tilemap::new());
        world
            .system_named::<&mut Tilemap>("DrawTilemap")
            .term_at(0)
            .singleton()
            .each(|tm| {
            //     let floor = game.textures["floor"];
            //     let wall = game.textures["wall"];
            //     for x in 0..tm.w {
            //         for y in 0..tm.h {
            //             let (fx, fy) = (x as f32 * 32., y as f32 * 32.);
            //             match tm[(x, y)] {
            //                 Tile::Floor => {
            //                     draw_texture(floor, fx, fy, WHITE);
            //                 }
            //                 Tile::Wall => {
            //                     draw_texture(wall, fx, fy, WHITE);
            //                 }
            //             }
            //         }
            //     }
            });
    }
}
