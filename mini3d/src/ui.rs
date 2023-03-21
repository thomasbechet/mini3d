pub mod event;
pub mod controller;
pub mod user;
pub mod widget;

// macro_rules! define_add {
//     ($name:ident, $fname:ident, $widget:ident) => {
//         pub fn $fname(&mut self, name: &str, z_index: i32, parent: UID, $name: $widget) -> Result<UID> {
//             let uid = UID::new(name);
//             if self.widgets.contains_key(&uid) { return Err(anyhow!("Widget already exists")); }
//             self.widgets.insert(uid, Widget { z_index, parent, variant: WidgetVariant::$widget($name) });
//             Ok(uid)
//         }
//     };
// }

// macro_rules! define_get {
//     ($name:ident, $fname:ident, $widget:ident) => {
//         pub fn $fname(&self, uid: UID) -> Result<&$widget> {
//             let widget = self.widgets.get(&uid).with_context(|| "Widget not found")?;
//             match &widget.variant {
//                 WidgetVariant::$widget(widget) => Ok(widget),
//                 _ => { Err(anyhow!("Widget is not a {}", stringify!($widget))) }
//             }
//         }
//     };
// }

// macro_rules! define_get_mut {
//     ($name:ident, $fname:ident, $widget:ident) => {
//         pub fn $fname(&mut self, uid: UID) -> Result<&mut $widget> {
//             let widget = self.widgets.get_mut(&uid).with_context(|| "Widget not found")?;
//             match &mut widget.variant {
//                 WidgetVariant::$widget(widget) => Ok(widget),
//                 _ => { Err(anyhow!("Widget is not a {}", stringify!($widget))) }
//             }
//         }
//     };
// }