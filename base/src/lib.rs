use flecs_ecs::core::World;
use game::GameComponents;
use persist::PersistModule;

pub mod game;
pub mod persist;
pub mod util;
pub mod vendored;
pub use flecs_ecs;

pub fn register_components(world: &World) {
    world.import::<PersistModule>();
    world.import::<GameComponents>();
}
