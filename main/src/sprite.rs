use anyhow::Result;
use base::flecs_ecs;
use base::flecs_ecs::prelude::*;
use base::util::flecs_extension::{KfWorldExtensions, QueryExtKf};
use base::util::pos::Pos;
use base::util::vec2f::Vec2f;
use flecs::pipeline::{OnLoad, OnStore, PreStore};
use graphic::egui;
use graphic::macroquad::prelude::*;
use graphic::vendored::egui_macroquad::egui;
use std::collections::HashMap;

use crate::camera::{CameraComponents, CameraWrapper};
use crate::{FloorSprite, GameComponents, Player, TilemapComponents, Visible, WallSprite};
use base::game::Unit;

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

impl Into<Vec2> for &DrawPos {
    fn into(self) -> Vec2 {
        Vec2 {
            x: self.x,
            y: self.y,
        }
    }
}

impl Into<Vec2f> for &DrawPos {
    fn into(self) -> Vec2f {
        Vec2f {
            x: self.x,
            y: self.y,
        }
    }
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
        world.component_kf::<DrawPos>().meta();
        world.component_kf::<Sprite>();
        world.component_kf::<TextureStore>();
        world.component_kf::<FloorSprite>();
        world.component_kf::<WallSprite>();
    }
}

#[derive(Component)]
pub struct SpriteSystems {}

impl Module for SpriteSystems {
    fn module(w: &World) {
        w.import::<SpriteComponents>();
        w.import::<GameComponents>();
        w.import::<TilemapComponents>();
        w.import::<CameraComponents>();

        w.system::<&Pos>()
            .without::<DrawPos>()
            .each_entity(|e, _pos| {
                e.set(DrawPos::default()); // will be updated in same frame
            });
        w.system::<(&Pos, &mut DrawPos)>()
            .with::<Unit>()
            .kind::<PreStore>()
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

        w.system_named::<(&CameraWrapper, &DrawPos, &Unit)>("HoverUnitSystem")
            .term_singleton(0)
            .with::<Visible>()
            .each(|(camera, dp, unit)| {
                let mp = camera.screen_to_world(Vec2f::from(mouse_position()));
                let ordered = |a, b, c| (a <= b) && (b < c);
                let mouse_hovered =
                    ordered(dp.x, mp.x, dp.x + 32.0) && ordered(dp.y, mp.y, dp.y + 32.0);
                if mouse_hovered {
                    let label_pos = camera.world_to_screen(dp.into()) + Vec2f { x: 10., y: 20. };
                    egui::Area::new(egui::Id::new("hover_unit_area"))
                        .fixed_pos(egui::pos2(label_pos.x, label_pos.y))
                        .show(egui(), |ui| {
                            egui::Frame::none()
                                .fill(egui::Color32::BLACK)
                                .show(ui, |ui| {
                                    ui.label("Name:");
                                    ui.label(&unit.name);
                                });
                        });
                }
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
mod test {}
