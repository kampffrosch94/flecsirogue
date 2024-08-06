use flecs_ecs::prelude::*;

use crate::EguiContext;

#[derive(Component, Debug)]
pub struct Player;

#[derive(Component, Debug)]
pub struct Unit {
    pub name: String,
    pub health: Health,
}

#[derive(Debug, Clone)]
pub struct Health {
    pub max: i32,
    pub current: i32,
}

#[derive(Component, Default)]
pub struct MessageLog {
    pub messages: Vec<String>,
}

#[derive(Component)]
pub struct GameModule {}

impl Module for GameModule {
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
