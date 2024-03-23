use mini3d_db::database::Database;

use crate::api::API;

pub trait SystemParam: 'static {
    fn resolve(db: &Database) -> Self;
}

pub trait System {
    fn resolve(&mut self, db: &Database);
    fn run(&self, api: &mut API);
}

pub struct FunctionSystem<Params, F> {
    function: F,
    params: Option<Params>,
}

macro_rules! impl_system {
    (
        $(
            $($params:ident),+
        )?
    ) => {
        impl<
            F: Fn(
                &mut API,
                $($(&$params),+)?
            )
            $(,$($params: SystemParam),+)?
        > System for FunctionSystem<($( $($params,)+ )?), F> {
            #[allow(non_snake_case)]
            #[allow(unused)]
            fn resolve(&mut self, db: &Database) {
                self.params = Some(($($($params::resolve(db),)+)?));
            }
            #[allow(non_snake_case)]
            #[allow(unused)]
            fn run(&self, api: &mut API) {
                 let ($($($params,)+)?) = self.params.as_ref().unwrap();
                 (self.function)(api, $( $(($params),)+ )?);
            }
        }
    }
}

impl_system!();
impl_system!(T1);
impl_system!(T1, T2);
impl_system!(T1, T2, T3);
impl_system!(T1, T2, T3, T4);
impl_system!(T1, T2, T3, T4, T5);
impl_system!(T1, T2, T3, T4, T5, T6);
impl_system!(T1, T2, T3, T4, T5, T6, T7);
impl_system!(T1, T2, T3, T4, T5, T6, T7, T8);

pub trait IntoSystem<Params> {
    type System: System + 'static;
    fn into_system(self) -> Self::System;
}

macro_rules! impl_into_system {
    (
        $($(
            $params:ident
        ),+)?
    ) => {
        impl<F: Fn(&mut API, $($(&$params),+)?) + 'static $(, $($params: SystemParam),+ )?> IntoSystem<( $($($params,)+)? )> for F {
            type System = FunctionSystem<( $($($params,)+)? ), F>;

            fn into_system(self) -> Self::System {
                FunctionSystem {
                    function: self,
                    params: Default::default(),
                }
            }
        }
    }
}

impl_into_system!();
impl_into_system!(T1);
impl_into_system!(T1, T2);
impl_into_system!(T1, T2, T3);
impl_into_system!(T1, T2, T3, T4);
impl_into_system!(T1, T2, T3, T4, T5);
impl_into_system!(T1, T2, T3, T4, T5, T6);
impl_into_system!(T1, T2, T3, T4, T5, T6, T7);
impl_into_system!(T1, T2, T3, T4, T5, T6, T7, T8);
