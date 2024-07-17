use crate::util::vec2f::Vec2f;
use flecs_ecs::prelude::*;
use macroquad::prelude::*;
use tween::{Linear, Tweener};

const DPI_FACTOR: f32 = 2.0;

#[derive(Component)]
pub struct CameraWrapper {
    pub scale: f32,
    pub scale_exp: i32,
    pub offset: Vec2f,
    pub scale_tween: Tweener<f32, f32, Linear>,
    pub offset_tween: Tweener<Vec2f, f32, Linear>,
    pub camera: Camera2D,
}

impl CameraWrapper {
    pub fn new() -> Self {
        let scale_exp = 1;
        let base2: f32 = 2.;
        let scale = base2.powf(scale_exp as f32);
        let scale_tween = Tweener::linear(2., 2., 1.);

        let offset = Vec2f { x: 0., y: 0. };
        let offset_tween = Tweener::linear(offset, Vec2f { x: 32., y: 32. }, 0.25);

        let camera = Camera2D {
            zoom: vec2(scale / screen_width(), scale / screen_height()),
            rotation: 0.,
            offset: vec2(0., 0.0),
            target: offset.into(),
            render_target: None,
            viewport: None,
        };

        CameraWrapper {
            scale,
            scale_exp,
            scale_tween,
            offset,
            offset_tween,
            camera,
        }
    }

    pub fn set(&self) {
        set_camera(&self.camera);
    }

    /// do tweening and stuff
    pub fn process(&mut self) {
        // handle camera
        let mouse_position = Vec2f::from(mouse_position());

        if !self.offset_tween.is_finished() {
            self.offset = self.offset_tween.move_by(get_frame_time());
        }

        if !self.scale_tween.is_finished() {
            let point = Vec2f::from(self.camera.screen_to_world(mouse_position.into()));
            let new_scale = self.scale_tween.move_by(get_frame_time());
            let new_camera = Camera2D {
                zoom: vec2(new_scale / screen_width(), -new_scale / screen_height()),
                rotation: 0.,
                offset: vec2(0., 0.0),
                target: vec2(
                    screen_width() + self.offset.x,
                    screen_height() + self.offset.y,
                ),
                render_target: None,
                viewport: None,
            };
            let new_point = Vec2f::from(new_camera.screen_to_world(mouse_position.into()));
            let pan_correction = new_point - point;
            self.offset -= pan_correction;
            self.scale = new_scale;
        }

        self.camera = Camera2D {
            zoom: vec2(self.scale / screen_width(), -self.scale / screen_height()),
            rotation: 0.,
            offset: vec2(0., 0.0),
            target: vec2(
                screen_width() + self.offset.x,
                screen_height() + self.offset.y,
            ),
            render_target: None,
            viewport: None,
        };

        self.set();
    }

    pub fn mouse_delta(&mut self, delta: Vec2f) {
        self.offset += delta / (self.scale / DPI_FACTOR);
    }

    pub fn zoom(&mut self, delta: i32) {
        self.scale_exp += delta;
        let base2: f32 = 2.;
        self.scale_exp = self.scale_exp.clamp(1, 5);
        let target = base2.powf(self.scale_exp as f32);
        self.scale_tween = Tweener::linear(self.scale, target, 0.25);
    }

    pub fn move_camera(&mut self, (x, y): (f32, f32)) {
        self.offset.x += x;
        self.offset.y += y;
    }
}

#[derive(Component)]
pub struct CameraModule {}

impl Module for CameraModule {
    fn module(world: &flecs_ecs::prelude::World) {
        world.set(CameraWrapper::new());
        world
            .system_named::<&mut CameraWrapper>("CameraSystem")
            .term_at(0)
            .singleton()
            .each(|cw| cw.process());
    }
}
