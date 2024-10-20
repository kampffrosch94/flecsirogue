use flecs_ecs::prelude::*;
use nanoserde::{DeJson, SerJson};

use crate::{
    persist::{PersistExtension, PersistTagExtension},
    util::pos::Pos,
    EguiContext,
};

#[derive(Component, Debug, DeJson, SerJson, Default)]
#[meta]
pub struct Player {}

#[derive(Component, Debug, DeJson, SerJson)]
#[meta]
pub struct Unit {
    pub name: String,
    pub health: Health,
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
pub struct GameComponents {}

impl Module for GameComponents {
    fn module(world: &World) {
        world.module::<GameComponents>("GameComponents");
        world.component::<Pos>().meta().persist();
        world.component::<Player>().meta().persist();
        world.component::<Health>().meta().persist();
        world.component::<Unit>().meta().persist();
        world.component::<MessageLog>().persist();
        world.component::<EguiContext>();
    }
}

#[derive(Component)]
pub struct GameSystems {}

impl Module for GameSystems {
    fn module(world: &World) {
        world.set(MessageLog::default());
        world
            .system_named::<(&mut MessageLog, &Unit)>("UnitRemoveDead")
            .term_at(0)
            .singleton()
            .each_entity(|entity, (ml, unit)| {
                if unit.health.current <= 0 {
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
