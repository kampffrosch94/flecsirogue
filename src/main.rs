mod camera;
mod game;
mod input;
mod persist;
mod sprite;
mod tilemap;
mod util;
mod vendored;

use game::*;
use input::InputSystems;
use nanoserde::{DeJson, SerJson};
use persist::{Persist, Persister};
use sprite::*;
use tilemap::*;
use util::pos::Pos;
use vendored::*;

use camera::{CameraComponents, CameraSystems};

use flecs_ecs::prelude::*;
use macroquad::prelude::*;
use macroquad::rand::ChooseRandom;

fn window_conf() -> Conf {
    Conf {
        window_title: "Flecsirogue".to_owned(),
        fullscreen: false,
        high_dpi: true,
        ..Default::default()
    }
}

// we use this again on loading saves
async fn create_world() -> World {
    // not sure how to move the TextureStore into a module since it uses async for loading
    // resources
    let mut store = TextureStore::default();
    store
        .load_texture("assets/32rogues/rogues.png", "rogues")
        .await
        .unwrap();
    store
        .load_texture("assets/32rogues/tiles.png", "tiles")
        .await
        .unwrap();
    store
        .load_texture("assets/32rogues/monsters.png", "monsters")
        .await
        .unwrap();

    let world = World::new();

    world.component::<Persist>();
    world.component::<Persister>();

    world.import::<SpriteComponents>();
    world.import::<GameComponents>();
    world.import::<CameraComponents>();
    world.import::<TilemapComponents>();

    world.import::<SpriteSystems>();
    world.import::<GameSystems>();
    world.import::<CameraSystems>();
    world.import::<InputSystems>();
    world.import::<TilemapSystems>();

    let floor_s = FloorSprite {
        texture: store.get("tiles"),
        params: DrawTextureParams {
            source: Some(Rect::new(64., 416., 32., 32.)),
            ..Default::default()
        },
    };
    let wall_s = WallSprite {
        lower: Sprite {
            texture: store.get("tiles"),
            params: DrawTextureParams {
                source: Some(Rect::new(32., 160., 32., 32.)),
                ..Default::default()
            },
        },
        upper: Sprite {
            texture: store.get("tiles"),
            params: DrawTextureParams {
                source: Some(Rect::new(0., 160., 32., 32.)),
                ..Default::default()
            },
        },
    };

    world.set(floor_s);
    world.set(wall_s);
    world.set(store);

    // Creates REST server on default port (27750)
    // TODO need to turn these off before reloading world
    //world.import::<stats::Stats>())); // stats for explorer
    //world.set(flecs::rest::Rest::default());

    return world;
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = create_world().await;

    let player = world
        .entity_named("PlayerCharacter")
        .set(Unit {
            name: "Player".into(),
            health: Health {
                max: 10,
                current: 10,
            },
        })
        .add::<Player>();

    let mut free_positions = Vec::new();
    world.query::<&TileMap>().singleton().build().each(|tm| {
        for x in 0..tm.w {
            for y in 0..tm.h {
                let pos = Pos::new(x, y);
                if tm[pos] == TileKind::Floor {
                    free_positions.push(pos);
                }
            }
        }
    });

    // TODO think about reproduceable seeds
    free_positions.shuffle();

    player.set(free_positions.pop().unwrap());
    // place enemies
    for _ in 0..10 {
        world
            .entity()
            .set(Unit {
                name: "Goblin".into(),
                health: Health { max: 3, current: 3 },
            })
            .set(free_positions.pop().unwrap());
    }

    let mut backup = None;

    loop {
        clear_background(BLACK);

        if is_key_pressed(KeyCode::F5) {
            let s = persist::serialize_world(&world).serialize_json();
            backup = Some(s);
        }
        if is_key_pressed(KeyCode::F9) {
            if let Some(ref json) = backup {
                let new_world = create_world().await;
                let ds = Vec::deserialize_json(json).unwrap();
                persist::deserialize_world(&new_world, &ds);
                world = new_world;
                println!("World reloaded!");
            }
        }

        // unfortunately we can not call this method twice without completely refactoring
        // egui macroquad, so we wrap it around w.progress()
        egui_macroquad::ui(|egui_ctx| {
            let wrapper = EguiContext {
                // UNSAFE: we extend the liftetime to 'static so that
                // we can store the reference in a singleton
                // do not forgot to remove it before the egui context goes out of scope
                ctx: unsafe { std::mem::transmute(egui_ctx) },
            };
            world.set(wrapper);
            world.progress();
            world.remove::<EguiContext>();
        });

        // println!("{}", player.to_json(None));
        egui_macroquad::draw();
        next_frame().await
    }
}

#[derive(Component)]
pub struct EguiContext {
    pub ctx: &'static egui::Context,
}

#[cfg(test)]
mod test {
    #![allow(unused)]
    use flecs_ecs::prelude::*;
    use json::{FromJsonDesc, WorldToJsonDesc};

    #[derive(Component)]
    #[meta]
    struct Pos {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Component)]
    #[meta]
    pub struct Position {
        pub x: f32,
        pub y: f32,
    }

    #[test]
    fn goblin_test() {
        #[derive(Component, Debug)]
        struct MaxHealth(i32);

        let w = World::new();
        w.component::<MaxHealth>()
            .add_trait::<(flecs::OnInstantiate, flecs::Inherit)>();

        let goblin = w.prefab().set(MaxHealth(4));
        let e = w.entity().is_a_id(goblin);

        e.get::<&MaxHealth>(|mh| assert_eq!(4, mh.0));
        goblin.get::<&mut MaxHealth>(|mh| mh.0 = 5);
        e.get::<&MaxHealth>(|mh| assert_eq!(5, mh.0));
    }
}
