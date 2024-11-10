use base::{game::*, util::flecs_extension::QueryExtKf};
use flecs::pipeline::{OnValidate, PostUpdate};
use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct GameSystems {}

impl Module for GameSystems {
    fn module(world: &World) {
        world.import::<GameComponents>();

        world
            .system_named::<(
                &DamageEvent,
                &DamageKind,
                &mut Health,
                &Unit,
                &mut MessageLog,
            )>("DamageEvent processing")
            .kind::<PostUpdate>()
            .with_first_name::<DamageKind>("$kind")
            .with_first_name::<Target>("$target")
            .term_src(1, "$kind")
            .term_src(2, "$target")
            .term_src(3, "$target")
            .term_singleton(4)
            .each_entity(|e, (ev, kind, t_hp, t_unit, ml)| {
                println!("Processing {e:?}");
                let name = &t_unit.name;
                let amount = &ev.amount;
                ml.messages
                    .push(format!("{name} takes {amount} {kind} damage."));
                // TODO not only units should be able to take damage
                // TODO damage resistance
                t_hp.current -= ev.amount;
            });

        // check that DamageEvents are wellformed
        world
            .system_named::<&DamageEvent>("DamageEvent sanity check")
            .scope_open()
            .not()
            .with_enum_wildcard::<DamageKind>()
            .with_first_name::<Target>("$target")
            .with::<Health>()
            .set_src_name("$target")
            .with_first::<Origin>(*flecs::Any)
            .scope_close()
            .kind::<OnValidate>()
            .each_entity(|e, _| {
                panic!("DamageEvent is malformed:\n{e:?}");
            });
        world
            .system_named::<&DamageEvent>("DamageEvent cleanup")
            .kind::<PostUpdate>()
            .each_entity(|e, _| {
                println!("Deleting {e:?}");
                e.destruct();
            });

        world
            .system_named::<(&mut MessageLog, &Unit, &Health)>("UnitRemoveDead")
            .term_singleton(0)
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
    use base::game::DamageKind;

    use super::*;

    #[test]
    fn damage_event_test() {
        let world = World::new();
        world.import::<GameSystems>();

        let player = world.entity_named("player");
        let enemy = world
            .entity_named("gobbo")
            .set(Health { max: 5, current: 5 })
            .set(Unit {
                name: "Goblin McGobbo".into(),
            });
        let enemy2 = world
            .entity_named("gobbo 2")
            .set(Health { max: 5, current: 5 })
            .set(Unit {
                name: "Goblina McGobbo".into(),
            });
        world
            .entity()
            .set(DamageEvent { amount: 2 })
            .add_first::<Target>(enemy)
            .add_first::<Target>(enemy2)
            .add_first::<Origin>(player)
            .add_enum(DamageKind::Cutting);

        world.progress();
        assert_eq!(3, enemy.get::<&Health>(|hp| hp.current));
        assert_eq!(3, enemy2.get::<&Health>(|hp| hp.current));
    }
}
