use zed_extension_api::{Extension, register_extension};

struct Sql;

impl Extension for Sql {
    fn new() -> Self
    where
        Self: Sized,
    {
        Self
    }
}

register_extension!(Sql);
