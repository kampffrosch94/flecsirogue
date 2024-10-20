use flecs_ecs::prelude::*;

#[derive(Component)]
pub struct Persist {}


mod test {
    #![allow(unused)]
    use flecs_ecs::prelude::*;
    use super::*;

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

    #[test]
    fn serialize_entity() {
        let world = World::new();
        world.component::<Persist>();
        world.component::<Opaque>();
        world.component::<Transparent>().meta().add::<Persist>();
        let e = world
            //.entity_named("thing")
            .entity()
            .set(Opaque { stuff: 32 })
            .set(Transparent { stuff: 42 });

	println!("------------");
	e.get::<&Transparent>(|_|{});
        e.each_component(|comp| {
            if comp.is_entity() {
		let ev = comp.entity_view();
                println!("comp: {}", ev.symbol());
		if ev.has::<Persist>() {
		    let fetched = FetchedId::new(*comp.id());
		    let json = world.to_json_dyn(fetched, unsafe{&*e.get_untyped(comp)});
		    println!("json: {}", json);
		}
            } else if comp.is_pair() {
                println!("Pair {} + {}", comp.first_id().symbol(), comp.second_id().symbol());
            } else {
                panic!("No idea what this is: {:?}", comp);
            }
        });
	println!("------------");
    }
}
