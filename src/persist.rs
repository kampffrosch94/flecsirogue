use std::collections::HashSet;

use flecs::meta::TypeSerializer;
use flecs_ecs::prelude::*;
use nanoserde::{DeJson, SerJson};

#[derive(Component)]
pub struct Persist {}

pub trait PersistExtension {
    fn persist(&self) -> EntityView;
}

pub trait PersistTagExtension {
    fn persist(&self) -> EntityView;
}

impl<T> PersistTagExtension for Component<'_, T>
where
    T: TagComponent,
{
    fn persist(&self) -> EntityView {
        self.add::<Persist>()
    }
}

impl<T> PersistExtension for Component<'_, T>
where
    T: ComponentId + DataComponent + DeJson + SerJson + ComponentType<Struct>,
{
    fn persist(&self) -> EntityView {
        self.set(Persister::new::<T>()).add::<Persist>()
    }
}

#[derive(Component)]
pub struct Persister {
    pub serializer: Box<fn(EntityView) -> String>,
    pub deserializer: Box<fn(EntityView, &str)>,
    // for relationship values
    pub second_serializer: Box<fn(EntityView, Entity) -> String>,
    pub second_deserializer: Box<fn(EntityView, Entity, &str)>,
}

impl Persister {
    pub fn new<T>() -> Self
    where
        T: ComponentId + DataComponent + DeJson + SerJson + ComponentType<Struct>,
    {
        let ser = |ev: EntityView| ev.get::<&T>(|t| t.serialize_json());
        let deser = |ev: EntityView, s: &str| {
            ev.set(T::deserialize_json(s).unwrap());
        };

        let second_ser = |ev: EntityView, first: Entity| {
            //println!("First  {:?}", first);
            //println!("Second {:?}", std::any::type_name::<T>());
            ev.get_ref_second::<T>(first).get(|t| t.serialize_json())
        };

        let second_deser = |ev: EntityView, first: Entity, s: &str| {
            ev.set_second(first, T::deserialize_json(s).unwrap());
        };

        Self {
            serializer: Box::new(ser),
            deserializer: Box::new(deser),
            second_serializer: Box::new(second_ser),
            second_deserializer: Box::new(second_deser),
        }
    }
}

pub fn serialize_world(world: &World) -> Vec<SerializedEntity> {
    let query = world
        .query::<()>()
        .with_name("$comp")
        .with::<Persist>()
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

    //println!("Looking up tags");
    for tag in &s.tags {
        let ev = world.lookup(&tag);
        e.add_id(ev.id());
    }

    //println!("Looking up components");
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
                let rel = world.lookup(rel);
                let target = world.lookup(&target);
                target.get::<&Persister>(|p| (p.second_deserializer)(e, rel.id(), json));
            }
        }
    }

    //println!("Done here.");
    e
}

fn serialize_entity(e: EntityView) -> SerializedEntity {
    let mut components = Vec::new();
    let mut pairs = Vec::new();
    let mut tags = Vec::new();

    e.each_component(|comp| {
        if comp.is_entity() {
            let ev = comp.entity_view();
            let name = ev.path().unwrap();
            // println!("comp: {}", name);
            if ev.has::<Persist>() {
                if comp.type_id() != 0 {
                    // not a tag
                    let json = ev.get::<&Persister>(|p| (p.serializer)(e));
                    components.push((name, json).into());
                } else {
                    tags.push(ev.path().unwrap());
                }
            }
        } else if comp.is_pair() {
            // println!("Pair {} + {}", comp.first_id().name(), comp.second_id().name());
            let rel = comp.first_id();
            let target = comp.second_id();
            if rel.has::<Persist>() {
                if target.has::<flecs_ecs::core::flecs::Component>() {
                    let json = target
                        .try_get::<&Persister>(|p| (p.second_serializer)(e, rel.id()))
                        .expect("Component should have a Persister registered");
                    let s = SerializedTarget::Component(json);
                    pairs.push((rel.path().unwrap(), target.path().unwrap(), s));
                    //println!("[{:?}]", ev2.archetype());
                } else {
                    let s = SerializedTarget::Entity(*target.id());
                    pairs.push((rel.path().unwrap(), target.name(), s));
                }
            }
        } else {
            panic!("No idea what this is: {:?}", comp);
        }
    });

    //println!("Done");

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

#[derive(Debug, SerJson, DeJson, PartialEq)]
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

    #[derive(Component, Debug, SerJson, DeJson)]
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

    #[derive(Component, Debug, SerJson, DeJson)]
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
        world.component::<Persister>();
        world.component::<Opaque>();
        world.component::<Transparent>().meta().persist();
        world.component::<SomeTag>().meta().persist();
        world.component::<SomeRel>().meta().persist();
        world.component::<Health>().meta().persist();
        world.component::<Unit>().meta().persist();
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
        assert_eq!(
            SerializedTarget::Component("{\"stuff\":52}".into()),
            serialized.pairs[0].2
        );
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
    fn nested_minimal() {
        let world = World::new();
        world.component::<Persist>();
        world.component::<Health>().meta();
        world.component::<Unit>().meta();
        let e = world.entity().set(Unit {
            name: "VillagerA".into(),
            health: Health { max: 5, current: 3 },
        });
        println!("{}", e.to_json(None));
    }

    #[test]
    fn persister_test() {
        let world = World::new();
        world.component::<Persist>();
        world.component::<Persister>();
        world.component::<Health>().meta().persist();

        world.component::<Unit>().meta().persist();
        let e = world.entity().set(Unit {
            name: "VillagerA".into(),
            health: Health { max: 5, current: 3 },
        });
        let s = serialize_world(&world).serialize_json();
        let ds = Vec::deserialize_json(&s).unwrap();
        let world2 = World::new();
        world2.component::<Persist>();
        world2.component::<Persister>();
        world2.component::<Health>().meta().persist();
        world2.component::<Unit>().meta().persist();
        deserialize_world(&world2, &ds);
        dbg!(serialize_world(&world2));
        println!("{s}");
    }

    #[test]
    fn check_pair_understading() {
        let world = create_test_world();
        let rel_target = world.entity_named("RelTarget");
        let e = world
            .entity_named("thing")
            .set(Opaque { stuff: 32 })
            .set(Transparent { stuff: 42 })
            .set_pair::<SomeRel, _>(Transparent { stuff: 52 })
            .add_first::<SomeRel>(rel_target)
            .add::<SomeTag>();

        dbg!(SomeRel::id(&world));
        dbg!(e
            .get_ref_second::<Transparent>(SomeRel::id(&world))
            .get(|t| t.stuff));
        dbg!(e
            .get_ref_second::<Transparent>(SomeRel::get_id(&world))
            .get(|t| t.stuff));
    }
}
