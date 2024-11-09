use flecs_ecs::core::{Component, ComponentId, QueryBuilderImpl, World};

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

pub trait QueryExtKf<'a> {
    fn with_relation<First: ComponentId>(&mut self, first: &'a str, second: &'a str) -> &mut Self;
    fn term_src(&mut self, index: u32, src: &'a str) -> &mut Self;
    fn term_singleton(&mut self, index: u32) -> &mut Self;
}

impl<'a, T> QueryExtKf<'a> for T
where
    T: QueryBuilderImpl<'a>,
{
    fn with_relation<First: ComponentId>(&mut self, first: &'a str, second: &'a str) -> &mut Self {
        self.with_first_name::<First>(second).set_src_name(first)
    }

    fn term_src(&mut self, index: u32, src: &'a str) -> &mut Self {
        self.term_at(index).set_src_name(src)
    }

    fn term_singleton(&mut self, index: u32) -> &mut Self {
        self.term_at(index).singleton()
    }
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
