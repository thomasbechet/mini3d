use crate::{container::FieldIndex, database::Database, field::FieldType};

pub trait NativeComponent {
    const NAME: &'static str;
    type Meta;
    type Ref;
    type Mut;
    type ViewRef;
    type ViewMut;
    fn register(ecs: &mut Database) {}
}

pub struct NativeField<T: FieldType>(pub(crate) FieldIndex, core::marker::PhantomData<T>);

// pub struct MyTestComponent {
//     field0: i32,
//     field1: [i32; 8],
// }

// pub struct MyTestComponentMeta {
//     pub field0: NativeField<i32>,
//     pub field1: NativeField<[i32; 8]>,
// }

// impl NativeComponent for MyTestComponent {
//     const NAME: &'static str = "my_test_component";
//     type Meta = ();
//     type Ref = ();
//     type Mut = ();
//     type ViewRef = ();
//     type ViewMut = ();
//     fn register(ecs: &mut Database) {
//         ecs.register_native(Self::NAME).unwrap();
//     }
// }
