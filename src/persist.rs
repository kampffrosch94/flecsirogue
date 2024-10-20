#![allow(unused)]
use flecs::meta::TypeSerializer;
use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct Persist {}

fn serialize_entity(e: EntityView) -> SerializedEntity {
    let world = e.world();

    let mut components = Vec::new();
    let mut pairs = Vec::new();
    let mut tags = Vec::new();

    e.each_component(|comp| {
        if comp.is_entity() {
            let ev = comp.entity_view();
            let name = ev.symbol();
            println!("comp: {}", name);
            if ev.has::<Persist>() {
		println!("[{:?}]", ev.archetype());
		if ev.has::<TypeSerializer>() {
		    let fetched = FetchedId::new(*comp.id());
		    let json = world.to_json_dyn(fetched, unsafe { &*e.get_untyped(comp) });
		    println!("json: {}", json);
		    components.push((name, json).into());
		} else {
		    tags.push(ev.symbol());
		}
            }
        } else if comp.is_pair() {
            println!(
                "Pair {} + {}",
                comp.first_id().symbol(),
                comp.second_id().symbol()
            );
            let ev1 = comp.first_id();
            let ev2 = comp.second_id();
            if ev1.has::<Persist>() && ev2.has::<Persist>() {
                let fetched = FetchedId::new(*comp.id());
                let json = world.to_json_dyn(fetched, unsafe { &*e.get_untyped(comp) });
		pairs.push((ev1.symbol(), ev2.symbol(), json));

                // let json1 = world.to_json_dyn(FetchedId::new(*ev1.id()), unsafe { &*e.get_untyped(ev1) });
                // let json2 = world.to_json_dyn(FetchedId::new(*ev2.id()), unsafe { &*e.get_untyped(ev2) });
                // panic!("TODO I am not sure how Pairs work yet"));
                // are pairs made out of arbitrary components? or always (Tag, NormalComponent)?
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

#[derive(Debug)]
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

#[derive(Debug)]
pub struct SerializedEntity {
    id: u64,
    name: String,
    components: Vec<SerializedComponent>,
    pairs: Vec<(String, String, String)>,
    tags: Vec<String>,
}

mod test {
    #![allow(unused)]
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
        return world;
    }

    #[test]
    fn serialize_entity_test() {
        let world = create_test_world();
        let e = world
            .entity_named("thing")
            //.entity()
            .set(Opaque { stuff: 32 })
            .set(Transparent { stuff: 42 })
	    .add::<SomeTag>();
        println!("------------");
        // e.get::<&Transparent>(|_| {});
        let serialized = serialize_entity(e);
        println!("------------");
        dbg!(serialized);
    }

    // TODO test for pairs
    // TODO test for ids
}
