use base::flecs_ecs;
use base::flecs_ecs::prelude::*;
use graphic::macroquad::prelude::*;

use base::game::{DamageEvent, DamageKind, MessageLog, Player, PushEvent};
use base::util::{flecs_extension::QueryExtKf, pos::Pos};

use crate::{TileKind, TileMap};

#[derive(Component)]
pub struct InputSystems {}

impl Module for InputSystems {
    fn module(world: &World) {
        // move player
        world
            .system_named::<(&TileMap, &mut MessageLog, &mut Pos)>("PlayerMovement")
            .term_singleton(0)
            .term_singleton(1)
            .with::<Player>()
            .each_entity(|player_ev, (tm, _ml, pos)| {
                if !(is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift)) {
                    let direction_keys = [
                        (KeyCode::Kp1, (-1, 1)),
                        (KeyCode::Kp2, (0, 1)),
                        (KeyCode::Kp3, (1, 1)),
                        (KeyCode::Kp4, (-1, 0)),
                        (KeyCode::Kp5, (0, 0)),
                        (KeyCode::Kp6, (1, 0)),
                        (KeyCode::Kp7, (-1, -1)),
                        (KeyCode::Kp8, (0, -1)),
                        (KeyCode::Kp9, (1, -1)),
                    ];
                    let mut new_pos = *pos;
                    for (key, dir) in direction_keys {
                        if is_key_pressed(key) {
                            new_pos += dir;
                        }
                    }

                    if new_pos != *pos {
                        // check that we do not hit ourselves
                        let is_floor = tm.terrain[new_pos] == TileKind::Floor;
                        let maybe_blocker = tm.units.get(&new_pos);
                        let not_blocked = maybe_blocker.is_none();
                        if is_floor && not_blocked {
                            *pos = new_pos;
                        }
                        if let Some(other_entity) = maybe_blocker {
                            DamageEvent::create(
                                &player_ev.world(),
                                DamageKind::Cutting,
                                2,
                                *player_ev,
                                &[*other_entity],
                            );
                            let world = player_ev.world();
                            let other_ev = world.entity_from_id(*other_entity);
                            let p_pos = player_ev.get::<&Pos>(|pos| *pos);
                            let t_pos = other_ev.get::<&Pos>(|pos| *pos);
                            PushEvent::create(
                                &world,
                                t_pos - p_pos,
                                1,
                                *player_ev,
                                &[*other_entity],
                            );
                        }
                    }
                }
            });
    }
}
