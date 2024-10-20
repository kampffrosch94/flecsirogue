use std::{collections::HashSet, ptr::null_mut};

use flecs::meta::TypeSerializer;
use flecs_ecs::{prelude::*, sys};
use nanoserde::{DeJson, SerJson};

// TODO
// [ ] Pairs
// [ ] extension function
// [ ] Persister with zero sized types

#[derive(Component)]
pub struct Persister {
    pub serializer: Box<fn(EntityView) -> String>,
    pub deserializer: Box<fn(EntityView, &str)>,
}

impl Persister {
    pub fn new<T>() -> Self
    where
        T: ComponentId + DataComponent + DeJson + SerJson,
    {
        let ser = |ev: EntityView| ev.get::<&T>(|t| t.serialize_json());
        let deser = |ev: EntityView, s: &str| {
            ev.set(T::deserialize_json(s).unwrap());
        };
        Self {
            serializer: Box::new(ser),
            deserializer: Box::new(deser),
        }
    }
}

pub fn serialize_world(world: &World) -> Vec<SerializedEntity> {
    let query = world
        .query::<()>()
        .with_name("$comp")
        .with::<Persister>()
        .set_src_name("$comp")
        .build();
    let mut es = HashSet::new(); // want to have all entities only once
    query.each_entity(|e, _| {
        es.insert(e.id());
    });

    es.into_iter()
        .map(|e| serialize_entity(e.entity_view(world)))
        .collect()
}

pub fn deserialize_world(world: &World, ses: &Vec<SerializedEntity>) {
    for se in ses.iter() {
        dbg!(se);
        deserialize_entity(world, se);
    }
}

fn deserialize_entity<'a>(world: &'a World, s: &SerializedEntity) -> EntityView<'a> {
    let e = world.make_alive(s.id);
    if !s.name.is_empty() {
        e.set_name(&s.name);
    }

    println!("Looking up tags");
    for tag in &s.tags {
        let ev = world.lookup(&tag);
        e.add_id(ev.id());
    }

    println!("Looking up components");
    for comp in &s.components {
        dbg!(comp);
        let comp_e = world.try_lookup(&comp.name).unwrap();
        comp_e.get::<&Persister>(|p| (p.deserializer)(e, &comp.value));
    }

    for (rel, target, kind) in &s.pairs {
        match kind {
            SerializedTarget::Entity(te) => {
                let target = world.make_alive(*te);
                let rel = world.lookup(rel);
                let pair = ecs_pair(*rel.id(), *target.id());
                e.add_id(pair);
            }
            SerializedTarget::Component(json) => {
                // FIXME use new Persister pipeline
                let rel = world.lookup(rel);
                let target = world.lookup(&target);
                let pair = ecs_pair(*rel.id(), *target.id());
                unsafe { sys::ecs_emplace_id(world.world_ptr_mut(), *e.id(), pair, null_mut()) };
                let data_location = e.get_untyped_mut(pair);
                world.from_json_id(target, data_location, &json, None);
            }
        }
    }

    println!("Done here.");
    e
}

fn serialize_entity(e: EntityView) -> SerializedEntity {
    let world = e.world();

    let mut components = Vec::new();
    let mut pairs = Vec::new();
    let mut tags = Vec::new();

    e.each_component(|comp| {
        if comp.is_entity() {
            let ev = comp.entity_view();
            let name = ev.path().unwrap();
            //println!("comp: {}", name);
            if ev.has::<Persister>() {
                //println!("[{:?}]", ev.archetype());
                if ev.has::<TypeSerializer>() {
                    let json = ev.get::<&Persister>(|p| (p.serializer)(e));
                    components.push((name, json).into());
                } else {
                    tags.push(ev.path().unwrap());
                }
            }
        } else if comp.is_pair() {
            //println!("Pair {} + {}", comp.first_id().name(), comp.second_id().name());
            let ev1 = comp.first_id();
            let ev2 = comp.second_id();
            if ev1.has::<Persister>() {
                // println!("Lets persist.");
                if ev2.has::<flecs_ecs::core::flecs::Component>() {
                    let pointer = e.get_untyped(comp);
                    let json = world.to_json_id(ev2, pointer);
                    let s = SerializedTarget::Component(json);
                    pairs.push((ev1.path().unwrap(), ev2.path().unwrap(), s));
                    //println!("[{:?}]", ev2.archetype());
                } else {
                    let s = SerializedTarget::Entity(*ev2.id());
                    pairs.push((ev1.path().unwrap(), ev2.name(), s));
                }
            }
        } else {
            panic!("No idea what this is: {:?}", comp);
        }
    });

    println!("Done");

    SerializedEntity {
        id: e.id().0,
        name: e.name(),
        components,
        pairs,
        tags,
    }
}

#[derive(Debug, SerJson, DeJson)]
pub struct SerializedComponent {
    name: String,
    value: String,
}

impl From<(String, String)> for SerializedComponent {
    fn from(v: (String, String)) -> Self {
        Self {
            name: v.0,
            value: v.1,
        }
    }
}

#[derive(Debug, SerJson, DeJson)]
enum SerializedTarget {
    Component(String),
    Entity(u64),
}

#[derive(Debug, SerJson, DeJson)]
pub struct SerializedEntity {
    id: u64,
    name: String,
    components: Vec<SerializedComponent>,
    pairs: Vec<(String, String, SerializedTarget)>,
    tags: Vec<String>,
}

#[cfg(test)]
mod test {
    #![allow(unused)]
    use crate::{Health, Unit};

    use super::*;
    use flecs_ecs::prelude::*;

    #[derive(Component, Debug)]
    pub struct Opaque {
        stuff: u32,
    }

    impl Drop for Opaque {
        fn drop(&mut self) {
            if self.stuff != 32 {
                panic!("I can't be dropped right now");
            }
        }
    }

    #[derive(Component, Debug)]
    #[meta]
    pub struct Transparent {
        stuff: u32,
    }

    #[derive(Component, Debug)]
    #[meta]
    pub struct SomeTag {}

    #[derive(Component, Debug)]
    #[meta]
    pub struct SomeRel {}

    fn create_test_world() -> World {
        let world = World::new();
        world.component::<Persist>();
        world.component::<Opaque>();
        world.component::<Transparent>().meta().add::<Persist>();
        world.component::<SomeTag>().meta().add::<Persist>();
        world.component::<SomeRel>().meta().add::<Persist>();
        world.component::<Health>().meta().add::<Persist>();
        world.component::<Unit>().meta().add::<Persist>();
        return world;
    }

    #[test]
    fn serialize_entity_test() {
        let world = create_test_world();

        let rel_target = world.entity_named("RelTarget");
        let e = world
            .entity_named("thing")
            .set(Opaque { stuff: 32 })
            .set(Transparent { stuff: 42 })
            .set_pair::<SomeRel, _>(Transparent { stuff: 52 })
            .add_first::<SomeRel>(rel_target)
            .add::<SomeTag>();
        println!("{}", e.to_json(None));
        println!("------------");
        let serialized = serialize_entity(e);
        println!("------------");
        dbg!(&serialized);
        let id = e.id();

        let world2 = create_test_world();
        println!("------------");
        let deserialized = deserialize_entity(&world2, &serialized);
        println!("[{:?}]", deserialized.archetype());
        println!("------------");
        dbg!(serialize_entity(deserialized));
        assert_eq!(42, deserialized.get::<&Transparent>(|t| t.stuff));
        assert_eq!(
            52,
            deserialized.get::<(&(SomeRel, Transparent),)>(|(tp,)| tp.stuff)
        );
    }

    #[test]
    fn quick_check_test() {
        let world = create_test_world();

        let rel_target = world.entity_named("RelTarget");
        let e = world
            .entity_named("thing")
            //.entity()
            .set(Opaque { stuff: 32 })
            .set(Transparent { stuff: 42 })
            .set_pair::<SomeRel, _>(Transparent { stuff: 52 })
            .add_first::<SomeRel>(rel_target)
            .add::<SomeTag>();
        assert_eq!(42, e.get::<&Transparent>(|t| t.stuff));
        assert_eq!(52, e.get::<(&(SomeRel, Transparent),)>(|(tp,)| tp.stuff));
    }

    #[test]
    fn serialize_world_test() {
        let world = create_test_world();

        let rel_target = world.entity_named("RelTarget").add::<SomeTag>();
        let e = world
            .entity_named("thing")
            .set(Opaque { stuff: 32 })
            .set(Transparent { stuff: 42 })
            .set_pair::<SomeRel, _>(Transparent { stuff: 52 })
            .add_first::<SomeRel>(rel_target)
            .add::<SomeTag>();

        let s = serialize_world(&world).serialize_json();
        let ds = Vec::deserialize_json(&s).unwrap();
        let world2 = create_test_world();
        deserialize_world(&world2, &ds);
        dbg!(serialize_world(&world2));
        println!("{s}");
    }

    #[test]
    fn serialize_world_nested_test() {
        let world = create_test_world();
        let e = world.entity().set(Unit {
            name: "VillagerA".into(),
            health: Health { max: 5, current: 3 },
        });
        println!("{}", e.to_json(None));

        let s = serialize_world(&world).serialize_json();
        let ds = Vec::deserialize_json(&s).unwrap();
        let world2 = create_test_world();
        deserialize_world(&world2, &ds);
        dbg!(serialize_world(&world2));
        println!("{s}");
    }

    #[test]
    fn nested_minimal() {
        let world = World::new();
        world.component::<Persist>();
        world.component::<Health>().meta().add::<Persist>();
        world.component::<Unit>().meta().add::<Persist>();
        let e = world.entity().set(Unit {
            name: "VillagerA".into(),
            health: Health { max: 5, current: 3 },
        });
        println!("{}", e.to_json(None));
    }

    #[test]
    fn persister_test() {
        let world = World::new();
        world.component::<Persister>();
        world
            .component::<Health>()
            .meta()
            .set(Persister::new::<Health>());
        world
            .component::<Unit>()
            .meta()
            .set(Persister::new::<Unit>());
        let e = world.entity().set(Unit {
            name: "VillagerA".into(),
            health: Health { max: 5, current: 3 },
        });
        let s = serialize_world(&world).serialize_json();
        let ds = Vec::deserialize_json(&s).unwrap();
        let world2 = World::new();
        world2.component::<Persister>();
        world2
            .component::<Health>()
            .meta()
            .set(Persister::new::<Health>());
        world2
            .component::<Unit>()
            .meta()
            .set(Persister::new::<Unit>());
        deserialize_world(&world2, &ds);
        dbg!(serialize_world(&world2));
        println!("{s}");
    }
}
