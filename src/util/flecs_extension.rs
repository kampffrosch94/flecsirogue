use flecs_ecs::core::{Component, ComponentId, World};

pub trait KfWorldExtensions {
    fn component_kf<'a, T: ComponentId>(&'a self) -> Component<'a, T::UnderlyingType>;
}

impl KfWorldExtensions for World {
    fn component_kf<'a, T: ComponentId>(&'a self) -> Component<'a, T::UnderlyingType> {
        self.component_named::<T>(short_type_name::<T>())
    }
}

pub fn short_type_name<T: 'static>() -> &'static str {
    std::any::type_name::<T>().split("::").last().unwrap()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn shorten_type_name() {
        struct Something {}
        let s = short_type_name::<Something>();
        assert_eq!("Something", s);
    }
}
