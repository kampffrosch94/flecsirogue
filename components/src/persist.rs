use std::collections::HashSet;

use flecs_ecs::prelude::*;
use nanoserde::{DeJson, SerJson};

use crate::util::flecs_extension::KfWorldExtensions;

#[derive(Component)]
pub struct PersistModule {}

impl Module for PersistModule {
    fn module(world: &World) {
        //world.module::<PersistModule>("persist");
        world.component_kf::<Persist>();
        world.component_kf::<Persister>();
    }
}

#[derive(Component)]
pub struct Persist {}

pub trait PersistExtension<COMP> {
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

impl<T, COMP> PersistExtension<COMP> for Component<'_, T>
where
    T: CreatePersister<COMP>,
    COMP: ECSComponentType, // Struct or Enum
{
    fn persist(&self) -> EntityView {
        self.set(T::create_persister()).add::<Persist>()
    }
}

#[derive(Component)]
pub struct Persister {
    pub serializer: Box<fn(EntityView, u64) -> String>,
    pub deserializer: Box<fn(EntityView, u64, &str)>,
}

trait CreatePersister<COMP> {
    fn create_persister() -> Persister;
}

impl<T> CreatePersister<Struct> for T
where
    T: ComponentId + DataComponent + DeJson + SerJson + ComponentType<Struct>,
{
    fn create_persister() -> Persister {
        let ser = |ev: EntityView, id: u64| {
            let comp: &T = unsafe { &*ev.get_untyped(id).cast() };
            comp.serialize_json()
        };
        let deser = |ev: EntityView, id: u64, s: &str| {
            let data = T::deserialize_json(s).unwrap();
            ev.set_id(data, id);
        };

        Persister {
            serializer: Box::new(ser),
            deserializer: Box::new(deser),
        }
    }
}

impl<T> CreatePersister<Enum> for T
where
    T: ComponentId + DataComponent + DeJson + SerJson + ComponentType<Enum> + EnumComponentInfo,
{
    fn create_persister() -> Persister {
        let ser = |ev: EntityView, _id: u64| ev.get::<&T>(|comp| comp.serialize_json());
        let deser = |ev: EntityView, _id: u64, s: &str| {
            let data = T::deserialize_json(s).unwrap();
            ev.add_enum(data);
        };

        Persister {
            serializer: Box::new(ser),
            deserializer: Box::new(deser),
        }
    }
}

pub fn serialize_world(world: &World) -> Vec<SerializedEntity> {
    let query = world
        .query_named::<()>("Serialize World Query")
        .expr("!ChildOf(self|up, flecs)")
        .with_name("$comp")
        .or()
        .with_first_id(*flecs::Wildcard, "$comp")
        .or()
        .with_second_id("$comp", *flecs::Wildcard)
        .with::<Persist>()
        .set_src_name("$comp")
        .without_name("flecs.meta.member") // not sure how access this via type, since its a C type
        .set_cached()
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
        let type_id = comp_e.id_view().type_id().id();
        comp_e.get::<&Persister>(|p| (p.deserializer)(e, *type_id, &comp.value));
    }

    for (rel_name, target_name, kind) in &s.pairs {
        match kind {
            SerializedPair::Entity(te) => {
                let target = world.make_alive(*te);
                let rel = world.lookup(rel_name);
                let pair = ecs_pair(*rel.id(), *target.id());
                e.add_id(pair);
            }
            SerializedPair::TagComponent(json) => {
                let rel = world.lookup(rel_name);
                let target = world.lookup(&target_name);
                let pair = ecs_pair(*rel.id(), *target.id());
                target.get::<&Persister>(|p| (p.deserializer)(e, pair, json));
            }
            SerializedPair::ComponentEntity(json, te) => {
                let rel = world.lookup(rel_name);
                let target = world.make_alive(*te);
                if !target_name.is_empty() {
                    target.set_name(target_name);
                }
                let pair = ecs_pair(*rel.id(), *target.id());
                rel.get::<&Persister>(|p| (p.deserializer)(e, pair, json));
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

    println!("{e}");
    e.each_component(|comp| {
        println!("Comp: {:?}", &comp);
        if comp.is_entity() {
            let ev = comp.entity_view();
            let name = ev.path().unwrap();
            // println!("comp: {}", name);
            if ev.has::<Persist>() {
                if comp.type_id() != 0 {
                    // not a tag
                    let json = ev.get::<&Persister>(|p| (p.serializer)(e, *comp.type_id().id()));
                    components.push((name, json).into());
                } else {
                    tags.push(ev.path().unwrap());
                }
            }
        } else if comp.is_pair() {
            println!(
                "Pair {} + {}",
                comp.first_id().name(),
                comp.second_id().name()
            );
            let rel = comp.first_id();
            let target = comp.second_id();
            let pair = ecs_pair(*rel.id(), *target.id());
            if rel.has::<Persist>() {
                if rel.id_view().type_id() != 0 {
                    assert!(
                        !target.has::<flecs_ecs::core::flecs::Component>(),
                        "Only either first or second can be a data component"
                    );
                    let json = rel
                        .try_get::<&Persister>(|p| (p.serializer)(e, pair))
                        .expect("Component should have a Persister registered");
                    let s = SerializedPair::ComponentEntity(json, *target.id());
                    pairs.push((rel.path().unwrap(), target.name(), s));
                } else if target.has::<flecs_ecs::core::flecs::Component>() {
                    let json = target
                        .try_get::<&Persister>(|p| (p.serializer)(e, pair))
                        .expect("Component should have a Persister registered");
                    let s = SerializedPair::TagComponent(json);
                    pairs.push((rel.path().unwrap(), target.path().unwrap(), s));
                } else {
                    let s = SerializedPair::Entity(*target.id());
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
enum SerializedPair {
    ComponentEntity(String, u64),
    TagComponent(String),
    Entity(u64),
}

#[derive(Debug, SerJson, DeJson)]
pub struct SerializedEntity {
    id: u64,
    name: String,
    components: Vec<SerializedComponent>,
    pairs: Vec<(String, String, SerializedPair)>,
    tags: Vec<String>,
}

#[cfg(test)]
mod test {
    use crate::game::{Health, Unit};

    use super::*;

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
            SerializedPair::TagComponent("{\"stuff\":52}".into()),
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
    fn persister_test() {
        let world = World::new();
        world.component::<Persist>();
        world.component::<Persister>();
        world.component::<Health>().meta().persist();

        world.component::<Unit>().meta().persist();
        let e = world
            .entity()
            .set(Unit {
                name: "VillagerA".into(),
            })
            .set(Health { max: 5, current: 3 });
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

    #[test]
    fn persist_rel_component_entity() {
        #[derive(Debug, SerJson, DeJson, Component)]
        struct Amount {
            amount: i32,
        }

        let world = World::new();
        world.component::<Persist>();
        world.component::<Persister>();
        world.component::<Amount>().persist();

        let player = world.entity_named("Player");
        let item = world.entity_named("Some Item");
        player.set_first(Amount { amount: 1 }, item);
        println!("{}", player.to_json(None));

        let s = serialize_world(&world).serialize_json();
        println!("{s}");
        assert_ne!("[]", s);
        let ds = Vec::deserialize_json(&s).unwrap();
        let world2 = World::new();
        world2.component::<Persist>();
        world2.component::<Persister>();
        world2.component::<Amount>().persist();
        deserialize_world(&world2, &ds);
        println!("Deserialized");
        let player = player.id().id_view(&world2).entity_view();
        let item = world2.entity_named("Some Item");
        assert_eq!(1, player.get_ref_first::<Amount>(item).get(|i| i.amount));
    }

    #[test]
    fn persist_enum() {
        #[derive(Debug, SerJson, DeJson, Component)]
        #[repr(C)]
        #[meta]
        enum Thing {
            Stone,
            Rock,
            Boulder,
            Pebble,
        }

        let world = World::new();
        world.component::<Persist>();
        world.component::<Persister>();
        world.component::<Thing>().meta().persist();

        let player = world.entity_named("Player").add_enum(Thing::Rock);

        let s = serialize_world(&world).serialize_json();
        println!("{s}");
        assert_ne!("[]", s);
        let ds = Vec::deserialize_json(&s).unwrap();
        let world2 = World::new();
        world2.component::<Persist>();
        world2.component::<Persister>();
        world2.component::<Thing>().persist();

        deserialize_world(&world2, &ds);
        println!("Deserialized");

        let player = player.id().id_view(&world2).entity_view();
        assert!(player.has_enum(Thing::Rock));
    }
}
