use crate::util::vec2f::Vec2f;
use flecs_ecs::prelude::*;
use macroquad::prelude::*;
use tween::{Linear, Tweener};

const DPI_FACTOR: f32 = 1.0;

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
        let offset_tween = Tweener::linear(offset, Vec2f { x: 0., y: 0. }, 0.25);

        let camera = Self::create_camera(scale, offset);
        CameraWrapper {
            scale,
            scale_exp,
            scale_tween,
            offset,
            offset_tween,
            camera,
        }
    }

    pub fn create_camera(scale: f32, offset: Vec2f) -> Camera2D {
        Camera2D {
            zoom: vec2(scale / screen_width(), scale / screen_height()),
            rotation: 0.,
            offset: vec2(0., 0.),
            target: vec2(
                screen_width() / scale + offset.x,
                screen_height() / scale + offset.y,
            ),
            render_target: None,
            viewport: None,
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
            let new_camera = Self::create_camera(new_scale, self.offset);
            let new_point = Vec2f::from(new_camera.screen_to_world(mouse_position.into()));
            let pan_correction = new_point - point;
            self.offset -= pan_correction;
            self.scale = new_scale;
        }

        self.camera = Self::create_camera(self.scale, self.offset);
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
        let mut last_mouse_position = Vec2f::from(mouse_position());
        world
            .system_named::<&mut CameraWrapper>("CameraInputMouse")
            .term_at(0)
            .singleton()
            .each(move |cw| {
                if is_mouse_button_down(MouseButton::Middle) {
                    let delta: Vec2f = Vec2f::from(mouse_position()) - last_mouse_position;
                    cw.mouse_delta(delta);
                }
                last_mouse_position = Vec2f::from(mouse_position());
                match mouse_wheel() {
                    (_x, y) => {
                        if y != 0. {
                            if y > 0. {
                                cw.zoom(1);
                            }
                            if y < 0. {
                                cw.zoom(-1);
                            }
                        }
                    }
                }
            });

        world
            .system_named::<&mut CameraWrapper>("CameraInputKeyboard")
            .term_at(0)
            .singleton()
            .each(|cw| {
                if is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift) {
                    if is_key_pressed(KeyCode::W) {
                        cw.move_camera((0., -32.));
                    }
                    if is_key_pressed(KeyCode::S) {
                        cw.move_camera((0., 32.));
                    }
                    if is_key_pressed(KeyCode::A) {
                        cw.move_camera((-32., 0.));
                    }
                    if is_key_pressed(KeyCode::D) {
                        cw.move_camera((32., 0.));
                    }
                }
            });
        world
            .system_named::<&mut CameraWrapper>("CameraProcessing")
            .term_at(0)
            .singleton()
            .each(|cw| cw.process());
    }
}
