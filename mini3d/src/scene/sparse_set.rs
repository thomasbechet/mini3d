// use std::{ops::DerefMut, slice};

// struct Entry<K, T> {
//     key: K,
//     pub value: T,
// }

// pub(crate) struct SparseSet<K, T> {
//     dense: Vec<Entry<K, T>>,
//     sparse: Vec<usize>,
// }

// impl<T> SparseSet<T> {

//     fn index(&self, key: usize) -> Option<usize> {
//         let index = self.sparse[key];
//         if index < self.len() {
//             let entry = &self.dense[index];
//             if entry.key == key {
//                 return Some(index);
//             }
//         }
//         None
//     }

//     pub(crate) fn with_capacity(capacity: usize) -> Self {
//         let mut sparse = Vec::with_capacity(capacity);
//         unsafe { sparse.set_len(capacity); }
//         Self {
//             dense: Vec::with_capacity(capacity),
//             sparse,
//         }
//     }

//     pub(crate) fn len(&self) -> usize {
//         self.dense.len()
//     }
//     pub fn capacity(&self) -> usize {
//         self.sparse.len()
//     }

//     pub(crate) fn get(&self, key: usize) -> Option<&T> {
//         if let Some(index) = self.index(key) {
//             Some(&self.dense[index].value)
//         } else {
//             None
//         }
//     }

//     pub(crate) fn get_mut(&mut self, key: usize) -> Option<&mut T> {
//         if let Some(index) = self.index(key) {
//             Some(&mut self.dense[index].value)
//         } else {
//             None
//         }
//     }

//     pub(crate) fn contains(&self, key: usize) -> bool {
//         self.index(key).is_some()
//     }

//     pub(crate) fn insert(&mut self, key: usize, value: T) -> bool {
//         if let Some(current_value) = self.get_mut(key) {
//             *current_value = value;
//             return false;
//         }
//         let n = self.dense.len();
//         self.dense.push(Entry { key, value });
//         self.sparse[key] = n;
//         true
//     }

//     pub(crate) fn remove(&mut self, key: usize) -> Option<T> {
//         if self.contains(key) {
//             let index = self.sparse[key];
//             let r = self.dense.swap_remove(index).value;
//             if index < self.len() {
//                 let swapped_entry = &self.dense[index];
//                 self.sparse[swapped_entry.key] = index;
//             }
//             self.sparse[key] = self.capacity();
//             Some(r)
//         } else {
//             None
//         }
//     }
// }

// impl<T> DerefMut for SparseSet<T> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.dense[..]
//     }
// }

// impl<T> IntoIterator for SparseSet<T> {
//     type Item = Entry<T>;
//     type IntoIter = std::vec::IntoIter<Self::Item>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.dense.into_iter()
//     }
// }

// impl<'a, T> IntoIterator for &'a SparseSet<T> {
//     type Item = &'a Entry<T>;
//     type IntoIter = slice::Iter<'a, Entry<T>>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter()
//     }
// }

// impl<'a, T> IntoIterator for &'a mut SparseSet<T> {
//     type Item = &'a mut Entry<T>;
//     type IntoIter = slice::IterMut<'a, Entry<T>>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.iter_mut()
//     }
// }
