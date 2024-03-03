#![no_std]

#[cfg(test)]
extern crate std;

extern crate alloc;

pub mod bitset;
pub mod container;
pub mod database;
pub mod entity;
pub mod error;
pub mod field;
pub mod query;
pub mod registry;

#[cfg(test)]
mod test {
    use std::println;

    use crate::database::Database;

    use crate::entity::Entity;
    use crate::field::{ComponentField, ComponentFieldCollection, ComponentFieldType};
    use crate::query::Query;

    #[derive(Default)]
    struct MyContext;

    #[test]
    fn test() {
        let mut db = Database::default();
        let tag = db.register_tag("tag").unwrap();
        let c0 = db
            .register(
                "c0",
                &[
                    ComponentField {
                        name: "f0",
                        ty: ComponentFieldType::Entity,
                        collection: ComponentFieldCollection::Scalar,
                    },
                    ComponentField {
                        name: "f1",
                        ty: ComponentFieldType::U32,
                        collection: ComponentFieldCollection::Scalar,
                    },
                ],
            )
            .unwrap();
        let f0 = db.find_field::<Entity>(c0, "f0").unwrap();
        let f1 = db.find_field::<u32>(c0, "f1").unwrap();
        let e = db.create();
        db.add_default(e, tag);
        db.add_default(e, c0);
        assert!(db.has(e, tag));
        db.remove(e, tag);
        assert!(!db.has(e, tag));

        db.write(e, f0, Entity::null());
        db.write(e, f1, 42);

        assert!(db.read(e, f0).unwrap() == Entity::null());
        assert!(db.read(e, f1).unwrap() == 42);

        for _ in 0..100 {
            let e = db.create();
            db.add_default(e, c0);
        }

        let q = Query::default().all(&[c0]);
        let q = db.query_entities(&q).into_iter(&db);
        for e in q {
            println!("e: {:?}", e);
        }
    }
}
