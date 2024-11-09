use flecs::pipeline::OnValidate;
use flecs_ecs::prelude::*;
use nanoserde::{DeJson, SerJson};

use crate::{
    persist::{PersistExtension, PersistModule, PersistTagExtension},
    util::{flecs_extension::KfWorldExtensions, pos::Pos},
    EguiContext,
};

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

#[derive(Component)]
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
    pub origin: Entity,
    pub target: Entity,
    pub amount: i32,
}

#[derive(Component)]
pub struct GameComponents {}

impl Module for GameComponents {
    fn module(world: &World) {
        world.import::<PersistModule>();
        //world.module::<GameComponents>("game");
        world.component_kf::<DamageKind>().meta();
        world.component_kf::<DamageEvent>().meta();
        world.component_kf::<Pos>().meta().persist();
        world.component_kf::<Player>().meta().persist();
        world.component_kf::<Health>().meta().persist();
        world.component_kf::<Unit>().meta().persist();
        world.component_kf::<MessageLog>().persist();
        world.component_kf::<EguiContext>();
    }
}

#[derive(Component)]
pub struct GameSystems {}

impl Module for GameSystems {
    fn module(world: &World) {
        world.import::<GameComponents>();
        //world.module::<GameSystems>("game_systems");
        world.set(MessageLog::default());

        // check that DamageEvents are wellformed
        world
            .system_named::<&DamageEvent>("DamageEvent sanity check")
            .without_enum_wildcard::<DamageKind>()
            .kind::<OnValidate>()
            .each_entity(|e, _| {
                panic!("DamageEvent is malformed:\n{e}");
            });
        world
            .system_named::<(&mut MessageLog, &Unit, &Health)>("UnitRemoveDead")
            .term_at(0)
            .singleton()
            .each_entity(|entity, (ml, unit, hp)| {
                if hp.current <= 0 {
                    ml.messages.push(format!("{} dies.", unit.name));
                    println!("Deleting an entitiy. {:?}", entity);
                    entity.destruct();
                }
            });
        world
            .system_named::<(&MessageLog, &EguiContext)>("EguiMessageLog")
            .term_at(0)
            .singleton()
            .term_at(1)
            .singleton()
            .each(|(ml, egui)| {
                egui::Window::new("Message Log").show(egui.ctx, |ui| {
                    for msg in &ml.messages {
                        ui.label(msg);
                    }
                });
            });
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn damage_test() {
        let world = World::new();
        world.import::<GameSystems>();

        let player = world.entity_named("player");
        let enemy = world
            .entity_named("gobbo")
            .set(Health { max: 5, current: 5 });
        world
            .entity()
            .set(DamageEvent {
                origin: *player,
                target: *enemy,
                amount: 2,
            })
            .add_enum(DamageKind::Cutting);

        world.progress();
        assert_eq!(3, enemy.get::<&Health>(|hp| hp.current));
    }
}
