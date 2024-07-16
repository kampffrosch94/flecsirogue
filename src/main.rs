use anyhow::Result;
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
struct Position {
    x: i32,
    y: i32,
}

#[derive(Component, Debug)]
struct Sprite {
    texture: Texture2D,
    x: f32,
    y: f32,
    params: DrawTextureParams,
}

#[derive(Component, Debug)]
struct Player;

#[derive(Component, Debug)]
struct Unit;

#[derive(Component, Debug)]
struct Terrain;

#[macroquad::main("Flecsirogue")]
async fn main() {
    // TODO look into CameraWrapper in my other macroquad project
    let scale = 4.0;
    let camera = Camera2D {
        zoom: vec2(scale / screen_width(), scale / screen_height()),
        rotation: 0.,
        offset: vec2(0., 0.0),
        target: vec2(screen_width() / scale, screen_height() / scale),
        render_target: None,
        viewport: None,
    };

    set_camera(&camera);

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
        .set(Position { x: 3, y: 3 })
        .add::<Player>()
        .set(Sprite {
            texture: store.get("rogues"),
            x: 0.,
            y: 0.,
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
                    x: 32.0 * x as f32,
                    y: 32.0 * y as f32,
                    params: DrawTextureParams {
                        source: Some(Rect::new(0., 32., 32., 32.)),
                        ..Default::default()
                    },
                })
                .add::<Terrain>();
        }
    }

    w.system::<(&Position, &mut Sprite)>()
        .with::<Unit>()
        .each(move |(pos, sprite)| {
            sprite.x = 32. * pos.x as f32;
            sprite.y = 32. * pos.y as f32;
        });
    w.system::<&Sprite>()
        .with::<Terrain>()
        .kind::<OnStore>()
        .each(move |sprite| {
            draw_texture_ex(
                &sprite.texture,
                sprite.x,
                sprite.y,
                WHITE,
                sprite.params.clone(),
            );
        });
    w.system::<&Sprite>()
        .with::<Unit>()
        .kind::<OnStore>()
        .each(move |sprite| {
            draw_texture_ex(
                &sprite.texture,
                sprite.x,
                sprite.y,
                WHITE,
                sprite.params.clone(),
            );
        });

    loop {
        clear_background(BLACK);

        player.get::<&mut Position>(|pos| {
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

        next_frame().await
    }
}
