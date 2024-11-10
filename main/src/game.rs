use base::flecs_ecs;
use base::flecs_ecs::prelude::*;
use base::util::flecs_extension::KfWorldExtensions;
use base::util::pos::Pos;
use base::{game::*, util::flecs_extension::QueryExtKf};
use flecs::pipeline::PostUpdate;
use graphic::vendored::egui_macroquad::egui;

#[derive(Component)]
pub struct EguiEnabled {}

#[derive(Component)]
pub struct GameSystems {}

impl Module for GameSystems {
    fn module(world: &World) {
        world.import::<GameComponents>();
        world.component_kf::<EguiEnabled>();

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
            .each(|(ev, kind, t_hp, t_unit, ml)| {
                //println!("Processing {e:?}");
                let name = &t_unit.name;
                let amount = &ev.amount;
                ml.messages
                    .push(format!("{name} takes {amount} {kind} damage."));
                // TODO not only units should be able to take damage
                // TODO damage resistance
                t_hp.current -= ev.amount;
            });

        world
            .system_named::<(&PushEvent, &Unit, &mut Pos, &mut MessageLog)>(
                "PushEvent processing",
            )
            .kind::<PostUpdate>()
            .with_first_name::<Target>("$target")
            .term_src(1, "$target")
            .term_src(2, "$target")
            .term_singleton(3)
            .each_entity(|e, (ev, t_unit, t_pos, ml)| {
                println!("Processing {e:?}");
                let name = &t_unit.name;
                ml.messages.push(format!("{name} gets pushed."));
                // TODO not only units should be able to take damage
                // TODO damage resistance
                *t_pos += ev.direction * ev.distance;
            });

        world
            .system_named::<()>("Event cleanup")
            .kind::<PostUpdate>()
            .with::<DamageEvent>()
            .or()
            .with::<PushEvent>()
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
            .system_named::<&MessageLog>("EguiMessageLog")
            .term_singleton(0)
            .with::<EguiEnabled>()
            .singleton()
            .each(|ml| {
                graphic::egui::Window::new("Message Log").show(egui(), |ui| {
                    for msg in &ml.messages {
                        ui.label(msg);
                    }
                });
            });
    }
}

#[cfg(test)]
mod test {
    use base::{game::DamageKind, util::pos::Pos};

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

        let ev = DamageEvent::create(&world, DamageKind::Cutting, 2, *player, &[*enemy, *enemy2]);

        world.progress();
        assert_eq!(3, enemy.get::<&Health>(|hp| hp.current));
        assert_eq!(3, enemy2.get::<&Health>(|hp| hp.current));
	assert!(!ev.is_alive());
    }

    #[test]
    fn push_event_test() {
        let world = World::new();
        world.import::<GameSystems>();

        let player = world.entity_named("player");
        let enemy = world
            .entity_named("gobbo")
            .set(Health { max: 5, current: 5 })
            .set(Unit {
                name: "Goblin McGobbo".into(),
            })
            .set(Pos { x: 3, y: 2 });
        let enemy2 = world
            .entity_named("gobbo 2")
            .set(Health { max: 5, current: 5 })
            .set(Unit {
                name: "Goblina McGobbo".into(),
            })
            .set(Pos { x: 0, y: 0 });

        let ev = PushEvent::create(&world, (1, 1).into(), 1, *player, &[*enemy, *enemy2]);

        world.progress();
        assert_eq!(Pos::new(4, 3), enemy.get::<&Pos>(|pos| *pos));
        assert_eq!(Pos::new(1, 1), enemy2.get::<&Pos>(|pos| *pos));
	assert!(!ev.is_alive());
    }
}
