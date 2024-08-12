mod camera;
mod game;
mod sprite;
mod tilemap;
mod util;
mod vendored;

use egui::Slider;
use game::{GameModule, Health, MessageLog, Player, Unit};
use sprite::*;
use tilemap::*;
use util::pos::Pos;
use vendored::*;

use camera::CameraModule;

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

#[macroquad::main(window_conf)]
async fn main() {
    let world = World::new();

    // Creates REST server on default port (27750)
    world.import::<stats::Stats>(); // stats for explorer
    world.set(flecs::rest::Rest::default());

    world.import::<GameModule>();
    world.import::<CameraModule>();
    world.import::<SpriteModule>();
    world.import::<TilemapModule>();

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
    let player = world
        .entity_named("Player")
        .set(Unit {
            name: "Player".into(),
            health: Health {
                max: 10,
                current: 10,
            },
        })
        .add::<Player>()
        .set(Sprite {
            texture: store.get("rogues"),
            params: DrawTextureParams {
                source: Some(Rect::new(0., 0., 32., 32.)),
                ..Default::default()
            },
        });

    let enemy_sprite = Sprite {
        texture: store.get("monsters"),
        params: DrawTextureParams {
            source: Some(Rect::new(0., 0., 32., 32.)),
            ..Default::default()
        },
    };

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


    world
        .system_named::<(&mut WallSprite, &mut FloorSprite, &EguiContext)>("SpriteSelector")
        .term_at(0)
        .singleton()
        .term_at(1)
        .singleton()
        .each(|(wall_s, floor_s, egui)| {
            egui::Window::new("Sprite selector").show(egui.ctx, |ui| {
                if let Some(ref mut rect) = wall_s.lower.params.source {
                    ui.label("wall sprite:");
                    ui.add(
                        Slider::new(&mut rect.x, 0.0..=640.0)
                            .text("x")
                            .step_by(32.0),
                    );
                    ui.add(
                        Slider::new(&mut rect.y, 0.0..=640.0)
                            .text("y")
                            .step_by(32.0),
                    );
                }

                if let Some(ref mut rect) = floor_s.params.source {
                    ui.label("floor sprite:");
                    ui.add(
                        Slider::new(&mut rect.x, 0.0..=640.0)
                            .text("x")
                            .step_by(32.0),
                    );
                    ui.add(
                        Slider::new(&mut rect.y, 0.0..=640.0)
                            .text("y")
                            .step_by(32.0),
                    );
                }
            });
        });

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
            .set(enemy_sprite.clone())
            .set(free_positions.pop().unwrap());
    }
    // move player
    world
        .system_named::<(&TileMap, &mut MessageLog, &mut Pos)>("PlayerMovement")
        .term_at(0)
        .singleton()
        .term_at(1)
        .singleton()
        .with::<Player>()
        .each_entity(|e, (tm, ml, pos)| {
            if !(is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)) {
                let direction_keys = [
                    (KeyCode::Kp1, (-1, 1)),
                    (KeyCode::Kp2, (0, 1)),
                    (KeyCode::Kp3, (1, 1)),
                    (KeyCode::Kp4, (-1, 0)),
                    (KeyCode::Kp5, (0, 0)),
                    (KeyCode::Kp6, (1, 0)),
                    (KeyCode::Kp7, (-1, -1)),
                    (KeyCode::Kp8, (0, -1)),
                    (KeyCode::Kp9, (1, -1)),
                ];
                let mut new_pos = *pos;
                for (key, dir) in direction_keys {
                    if is_key_pressed(key) {
                        new_pos += dir;
                    }
                }

                if new_pos != *pos {
                    // check that we do not hit ourselves
                    let is_floor = tm.terrain[new_pos] == TileKind::Floor;
                    let maybe_blocker = tm.units.get(&new_pos);
                    let not_blocked = maybe_blocker.is_none();
                    if is_floor && not_blocked {
                        *pos = new_pos;
                    }
                    if let Some(other_entity) = maybe_blocker {
                        let other = other_entity.entity_view(e);
                        other.get::<&mut Unit>(|unit| {
                            unit.health.current -= 2;
                            ml.messages.push(format!("You hit the {}.", unit.name));
                        });
                    }
                }
            }
        });

    loop {
        clear_background(BLACK);

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
            egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                ui.label("Test");
            });
            world.remove::<EguiContext>();
        });

        egui_macroquad::draw();
        next_frame().await
    }
}

#[derive(Component)]
pub struct EguiContext {
    pub ctx: &'static egui::Context,
}

#[test]
#[cfg(test)]
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

mod test {

    use flecs_ecs::prelude::*;
    #[test]
    #[cfg(test)]
    fn cursed_inheritance() {
        #[derive(Component, Debug)]
        struct Unit {
            name: String,
            health: i32,
        }
        #[derive(Component)]
        struct Player {}
        let w = World::new();
        w.component::<Player>().is_a::<Unit>();

        let _goblin = w.entity().set(Unit {
            health: 3,
            name: "Goblin".into(),
        });
        let _player = w.entity().add::<Player>();
        let _goblin = w.entity().set(Unit {
            health: 0,
            name: "Goblin".into(),
        });
        let _goblin = w.entity().set(Unit {
            health: 3,
            name: "Goblin".into(),
        });
        w.system::<&mut Unit>().each_entity(|entity, unit| {
            if unit.health <= 0 {
                println!("Destroying {:?}", entity);
                entity.destruct();
            }
        });
        for _ in 0..100000 {
            w.progress();
        }
    }
}
