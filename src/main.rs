mod camera;
mod tilemap;
mod util;
mod vendored;
use tilemap::TilemapModule;
use vendored::*;

use anyhow::Result;
use camera::CameraModule;
use flecs::pipeline::OnStore;
use std::{collections::HashMap, ops::Index};

use flecs_ecs::prelude::*;
use macroquad::prelude::*;

#[derive(Default)]
pub struct TextureStore {
    textures: HashMap<String, Texture2D>,
}

impl TextureStore {
    pub async fn load_texture(
        &mut self,
        path: impl AsRef<str>,
        name: impl Into<String>,
    ) -> Result<()> {
        let texture = load_texture(path.as_ref()).await?;
        texture.set_filter(FilterMode::Nearest);
        self.textures.insert(name.into(), texture);
        Ok(())
    }

    pub fn get(&self, name: impl AsRef<str>) -> Texture2D {
        self[name.as_ref()].clone()
    }
}

impl Index<&str> for TextureStore {
    type Output = Texture2D;

    fn index(&self, index: &str) -> &Self::Output {
        &self.textures[index]
    }
}

#[derive(Component, Debug)]
struct Pos {
    x: i32,
    y: i32,
}

#[derive(Component, Debug, Default)]
struct DrawPos {
    x: f32,
    y: f32,
}

#[derive(Component, Debug)]
struct Sprite {
    texture: Texture2D,
    params: DrawTextureParams,
}

#[derive(Component, Debug)]
struct Player;

#[derive(Component, Debug)]
struct Unit;

#[derive(Component, Debug)]
struct Terrain;

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
    // TODO look into CameraWrapper in my other macroquad project
    let scale = 8.0;

    let mut store = TextureStore::default();
    store
        .load_texture("assets/32rogues/rogues.png", "rogues")
        .await
        .unwrap();
    store
        .load_texture("assets/32rogues/tiles.png", "tiles")
        .await
        .unwrap();

    let w = World::new();
    w.component::<Player>().is_a::<Unit>();

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

    for x in 0..10 {
        for y in 0..10 {
            w.entity()
                .set(Sprite {
                    texture: store.get("tiles"),
                    params: DrawTextureParams {
                        source: Some(Rect::new(0., 32., 32., 32.)),
                        ..Default::default()
                    },
                })
                .set(DrawPos {
                    x: 32.0 * x as f32,
                    y: 32.0 * y as f32,
                })
                .add::<Terrain>();
        }
    }

    w.system::<&Pos>()
        .without::<DrawPos>()
        .each_entity(|e, pos| {
            e.set(DrawPos::default()); // will be updated in same frame
        });
    w.system::<(&Pos, &mut DrawPos)>()
        .with::<Unit>()
        .each(move |(pos, dpos)| {
            dpos.x = 32. * pos.x as f32;
            dpos.y = 32. * pos.y as f32;
        });
    w.system::<(&Sprite, &DrawPos)>()
        .with::<Terrain>()
        .kind::<OnStore>()
        .each(move |(sprite, dp)| {
            draw_texture_ex(&sprite.texture, dp.x, dp.y, WHITE, sprite.params.clone());
        });
    w.system::<(&Sprite, &DrawPos)>()
        .with::<Unit>()
        .kind::<OnStore>()
        .each(move |(sprite, dp)| {
            draw_texture_ex(&sprite.texture, dp.x, dp.y, WHITE, sprite.params.clone());
        });
    w.import::<CameraModule>();
    w.import::<TilemapModule>();

    loop {
        let camera = Camera2D {
            zoom: vec2(scale / screen_width(), scale / screen_height()),
            rotation: 0.,
            offset: vec2(0., 0.0),
            target: vec2(screen_width() / scale, screen_height() / scale),
            render_target: None,
            viewport: None,
        };
        set_camera(&camera);

        clear_background(BLACK);

        player.get::<&mut Pos>(|pos| {
            if !(is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)) {
                if is_key_pressed(KeyCode::W) {
                    pos.y -= 1;
                }
                if is_key_pressed(KeyCode::S) {
                    pos.y += 1;
                }
                if is_key_pressed(KeyCode::A) {
                    pos.x -= 1;
                }
                if is_key_pressed(KeyCode::D) {
                    pos.x += 1;
                }
            }
        });

        w.query::<&mut Sprite>()
            .with::<Terrain>()
            .build()
            .each(|sprite| {
                if let Some(ref mut r) = sprite.params.source {
                    if is_key_pressed(KeyCode::J) {
                        r.y += 32.0;
                    }
                    if is_key_pressed(KeyCode::K) {
                        r.y -= 32.0;
                    }
                    if is_key_pressed(KeyCode::L) {
                        r.x += 32.0;
                    }
                    if is_key_pressed(KeyCode::H) {
                        r.x -= 32.0;
                    }
                }
            });

        //draw_texture(&tileset, 50., 50., WHITE);

        let text = format!("{:?}", camera.screen_to_world(mouse_position().into()));
        draw_text(&text, 20.0, 20.0, 30.0, WHITE);

        w.progress();

        egui_macroquad::ui(|egui_ctx| {
            egui::Window::new("egui ❤ macroquad").show(egui_ctx, |ui| {
                ui.label("Test");
            });
        });

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
