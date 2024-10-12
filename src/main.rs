mod camera;
mod game;
mod input;
mod sprite;
mod tilemap;
mod util;
mod vendored;

use game::*;
use input::InputSystems;
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

#[derive(Component)]
pub struct Persist {}

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
    world.import::<stats::Stats>(); // stats for explorer
    world.set(flecs::rest::Rest::default());

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
            let query = world
                .query::<()>()
                .with_name("$comp")
                .with::<Persist>()
                .set_src_name("$comp")
                .build();

            let desc = json::IterToJsonDesc {
                serialize_entity_ids: true,
                serialize_values: true,
                serialize_fields: true,
                serialize_full_paths: true,
                serialize_type_info: true,
                //serialize_inherited: true,
                //serialize_builtin: true,
                //serialize_table:true,
                ..Default::default()
            };
            let query_string = query.to_json(Some(&desc)).unwrap();
            println!("{}", query_string);

            let s = world.to_json_world(None);
            //println!("{}", s);
            backup = Some(s);
        }
        if is_key_pressed(KeyCode::F9) {
            if let Some(ref json) = backup {
                let new_world = create_world().await;
                new_world.from_json_world(json, None);
                world = new_world;
                // BUG: sprites are serialized as null, even though they shouldn't be
                // possibly a flecs bug
                world.defer_begin();
                world.query::<&mut Sprite>().build().each_entity(|e, _| {
                    e.remove::<Sprite>();
                });
                world.defer_end();
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

mod test {
    use flecs_ecs::prelude::*;
    use json::{FromJsonDesc, WorldToJsonDesc};

    #[derive(Component)]
    #[meta]
    struct Pos {
        x: i32,
        y: i32,
    }
    #[test]
    fn serialization_test() {
        let world = World::new();
        world.component::<Pos>().meta();
        let e = world.entity();
        e.set(Pos { x: 5, y: 8 });
        let s = e.to_json(None);
        println!("{}", s);
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

    #[test]
    fn serde_relationship() {
        #[derive(Component)]
        #[meta]
        pub struct Eats;
        let world = World::new();
	world.component::<Eats>();

        // Entity used for Grows relationship
        let grows = world.entity_named("Grows");

        // Relationship objects
        let apples = world.entity_named("Apples");
        let pears = world.entity_named("Pears");

        // Create an entity with 3 relationships. Relationships are like regular components,
        // but combine two types/identifiers into an (relationship, object) pair.
        world
            .entity_named("Bob")
            // Pairs can be constructed from a type and entity
            .add_first::<Eats>(apples)
            .add_first::<Eats>(pears)
            // Pairs can also be constructed from two entity ids
            .add_id((grows, pears));

        let json = world.to_json_world(None);
        println!("{}", json);
    }

    #[test]
    fn world_serialisation_no_meta() {
        #[derive(Component, Debug)]
        pub struct Thing {
            stuff: u32,
        }
        let world = World::new();
        world.component::<Thing>();

        let e = world.entity().set(Thing { stuff: 32 });
        let s = e.to_json(None);
        println!("{}", s);
        let json = world.to_json_world(None);

        let world2 = World::new();
        world2.component::<Thing>();
        let desc: FromJsonDesc = FromJsonDesc {
            name: c"Test".as_ptr(),
            expr: c"Test".as_ptr(),
            lookup_action: None,
            lookup_ctx: unsafe { std::mem::transmute(std::ptr::null::<std::ffi::c_void>()) },
            strict: false,
        };
        world2.from_json_world(&json, Some(&desc));
        world2.new_query::<&Thing>().iterable().each(|thing| {
            dbg!(thing);
        });
    }

    #[test]
    fn world_serialisation_no_meta_minimal() {
        // notice how we are NOT adding #[meta] to this
        #[derive(Component, Debug)]
        pub struct Thing {
            stuff: u32,
        }
        let world = World::new();
        world.component::<Thing>();
        world.entity().set(Thing { stuff: 32 });
        let json = world.to_json_world(None);
        println!("{}", json);
	// Output:
	// {"results":[{"name":"#558", "id":558,
	// "components":{"flecsirogue.test.world_serialisation_no_meta_minimal.Thing":null}}]}

        let world2 = World::new();
        world2.component::<Thing>();
        world2.from_json_world(&json, None);
        world2.new_query::<&Thing>().iterable().each(|thing| {
            dbg!(thing); // prints some bogus data
            assert!(false); // fails
        });
    }
}
