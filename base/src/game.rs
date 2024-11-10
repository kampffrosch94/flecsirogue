use crate::util::{
    flecs_extension::KfWorldExtensions,
    pos::{Direction, Pos},
};
use derive_more::Display;
use flecs_ecs::prelude::*;
use nanoserde::{DeJson, SerJson};

use crate::persist::{PersistExtension, PersistModule, PersistTagExtension};

#[derive(Component, Debug, DeJson, SerJson, Default)]
#[meta]
pub struct Player {}

#[derive(Component, Debug, DeJson, SerJson)]
#[meta]
pub struct Unit {
    pub name: String,
}

#[derive(Debug, Clone, Component, DeJson, SerJson)]
#[meta]
pub struct Health {
    pub max: i32,
    pub current: i32,
}

#[derive(Component, Default, DeJson, SerJson)]
pub struct MessageLog {
    pub messages: Vec<String>,
}

#[derive(Component, Display)]
#[meta]
#[repr(C)]
pub enum DamageKind {
    Cutting,
    Blunt,
    Pierce,
    Fire,
}

#[derive(Component)]
#[meta]
pub struct DamageEvent {
    pub amount: i32,
}

impl DamageEvent {
    pub fn create<'a>(
        world: &'a World,
        kind: DamageKind,
        amount: i32,
        origin: Entity,
        targets: &[Entity],
    ) -> EntityView<'a> {
        let ev = world
            .entity()
            .set(Self { amount })
            .add_enum(kind)
            .add_first::<Origin>(origin);
        for target in targets {
            ev.add_first::<Target>(*target);
        }
        ev
    }
}

#[derive(Component)]
#[meta]
pub struct PushEvent {
    pub direction: Direction,
    pub distance: i32,
}

impl PushEvent {
    pub fn create<'a>(
        world: &'a World,
        direction: Direction,
        distance: i32,
        origin: Entity,
        targets: &[Entity],
    ) -> EntityView<'a> {
        let ev = world
            .entity()
            .set(Self {
                direction,
                distance,
            })
            .add_first::<Origin>(origin);
        for target in targets {
            ev.add_first::<Target>(*target);
        }
        ev
    }
}

#[derive(Component)]
#[meta]
/// Relation
/// (DamageEvent, Entity)
pub struct Target {}

#[derive(Component)]
#[meta]
/// Relation
/// (DamageEvent, Entity)
pub struct Origin {}

#[derive(Component)]
pub struct GameComponents {}

impl Module for GameComponents {
    fn module(world: &World) {
        world.import::<PersistModule>();

        world.component_kf::<Direction>().meta();
        world.component_kf::<Target>().meta();
        world.component_kf::<Origin>().meta();
        world.component_kf::<DamageKind>().meta();
        world.component_kf::<DamageEvent>().meta();
        world.component_kf::<PushEvent>().meta();
        world.component_kf::<Pos>().meta().persist();
        world.component_kf::<Player>().meta().persist();
        world.component_kf::<Health>().meta().persist();
        world.component_kf::<Unit>().meta().persist();
        world.component_kf::<MessageLog>().persist();
        world.set(MessageLog::default());
    }
}
