use anyhow::Result;
use flecs::pipeline::OnStore;
use flecs_ecs::prelude::*;
use macroquad::prelude::*;
use std::collections::HashMap;
use std::ops::Index;

use crate::util::pos::Pos;
use crate::Visible;

#[derive(Default, Component)]
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

#[derive(Component, Debug, Default)]
pub struct DrawPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone)]
pub struct Sprite {
    pub texture: Texture2D,
    pub params: DrawTextureParams,
}

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Unit;

#[derive(Component)]
pub struct SpriteModule {}

impl Module for SpriteModule {
    fn module(w: &World) {
        w.component::<Player>().is_a::<Unit>();

        w.system::<&Pos>()
            .without::<DrawPos>()
            .each_entity(|e, _pos| {
                e.set(DrawPos::default()); // will be updated in same frame
            });
        w.system::<(&Pos, &mut DrawPos)>()
            .with::<Unit>()
            .each(move |(pos, dpos)| {
                dpos.x = 32. * pos.x as f32;
                dpos.y = 32. * pos.y as f32;
            });
        w.system::<(&Sprite, &DrawPos)>()
            .with::<Visible>()
            .with::<Unit>()
            .kind::<OnStore>()
            .each(move |(sprite, dp)| {
                draw_texture_ex(&sprite.texture, dp.x, dp.y, WHITE, sprite.params.clone());
            });
    }
}
