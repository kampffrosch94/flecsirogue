use flecs_ecs::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub max: i32,
    pub current: i32,
}
