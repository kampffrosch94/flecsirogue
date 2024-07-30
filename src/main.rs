mod camera;
mod sprite;
mod tilemap;
mod util;
mod vendored;
use egui::Slider;
use sprite::*;
use tilemap::*;
use util::pos::Pos;
use vendored::*;

use camera::CameraModule;

use flecs_ecs::prelude::*;
use macroquad::prelude::*;

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
    let w = World::new();

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
    let player = w
        .entity_named("Player")
        .set(Pos { x: 3, y: 3 })
        .add::<Player>()
        .set(Sprite {
            texture: store.get("rogues"),
            params: DrawTextureParams {
                source: Some(Rect::new(0., 0., 32., 32.)),
                ..Default::default()
            },
        });

    let floor_s = FloorSprite {
        texture: store.get("tiles"),
        params: DrawTextureParams {
            source: Some(Rect::new(64., 416., 32., 32.)),
            ..Default::default()
        },
    };
    let wall_s = WallSprite {
        texture: store.get("tiles"),
        params: DrawTextureParams {
            source: Some(Rect::new(32., 160., 32., 32.)),
            ..Default::default()
        },
    };
    w.set(floor_s);
    w.set(wall_s);
    w.set(store);

    w.import::<CameraModule>();
    w.import::<SpriteModule>();
    w.import::<TilemapModule>();

    w.system_named::<(&mut WallSprite, &mut FloorSprite)>("SpriteSelector")
        .term_at(0)
        .singleton()
        .term_at(1)
        .singleton()
        .each(|(wall_s, floor_s)| {
            egui_macroquad::ui(|egui_ctx| {
                egui::Window::new("Sprite selector").show(egui_ctx, |ui| {
                    if let Some(ref mut rect) = wall_s.params.source {
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
        });

    loop {
        clear_background(BLACK);

        player.get::<&mut Pos>(|pos| {
            if !(is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)) {
                let mut dir = (0, 0);
                if is_key_pressed(KeyCode::W) {
		    dir = (0, -1);
                }
                if is_key_pressed(KeyCode::S) {
		    dir = (0, 1);
                }
                if is_key_pressed(KeyCode::A) {
		    dir = (-1, 0);
                }
                if is_key_pressed(KeyCode::D) {
		    dir = (1, 0);
                }
		*pos += dir;
            }
        });

        w.progress();

        // egui_macroquad::ui(|egui_ctx| {
        //     egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
        //         ui.label("Test");
        //     });
        // });

        egui_macroquad::draw();
        next_frame().await
    }
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
