use anyhow::Result;
use flecs::pipeline::{OnLoad, OnStore};
use flecs_ecs::prelude::*;
use macroquad::prelude::*;
use std::collections::HashMap;

use crate::game::Unit;
use crate::util::pos::Pos;
use crate::{FloorSprite, Player, Visible, WallSprite};

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
        self.textures[name.as_ref()].clone()
    }
}

#[derive(Component, Debug, Default)]
#[meta]
pub struct DrawPos {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Sprite {
    pub texture: Texture2D,
    pub params: DrawTextureParams,
}

#[derive(Component)]
pub struct SpriteComponents {}

impl Module for SpriteComponents {
    fn module(world: &World) {
        world.component::<DrawPos>().meta();
        world.component::<Sprite>();
        world.component::<TextureStore>();
        world.component::<FloorSprite>();
        world.component::<WallSprite>();
    }
}

#[derive(Component)]
pub struct SpriteSystems {}

impl Module for SpriteSystems {
    fn module(w: &World) {
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

        // TODO this should probably just be one entity that gets referenced from children
        // or something like that
        w.system_named::<&TextureStore>("CreateSpritesPlayer")
            .term_at(0)
            .singleton()
            .with::<Player>()
            .without::<&mut Sprite>()
            .kind::<OnLoad>()
            .each_entity(|e, store| {
                e.set(Sprite {
                    texture: store.get("rogues"),
                    params: DrawTextureParams {
                        source: Some(Rect::new(0., 0., 32., 32.)),
                        ..Default::default()
                    },
                });
            });

        w.system_named::<&TextureStore>("CreateSpritesUnit")
            .term_at(0)
            .singleton()
            .with::<Unit>()
            .without::<&mut Sprite>()
            .without::<Player>()
            .kind::<OnLoad>()
            .each_entity(|e, store| {
                e.set(Sprite {
                    texture: store.get("monsters"),
                    params: DrawTextureParams {
                        source: Some(Rect::new(0., 0., 32., 32.)),
                        ..Default::default()
                    },
                });
            });
    }
}

#[cfg(test)]
mod test {
    use crate::Sprite;
    use flecs_ecs::prelude::*;

    #[test]
    fn why_is_sprite_serialized() {
        let w = World::new();
        w.component::<Sprite>();
    }
}
