use glam::{Vec3, Vec2, Vec4};

type Word = u32;

pub(crate) struct Stack {
    stack: Vec<Word>,
}

impl Stack {

    pub(crate) fn new(capacity: usize) -> Self {
        Stack {
            stack: Vec::with_capacity(capacity),
        }
    }

    pub(crate) fn print(&self) {
        for word in &self.stack {
            println!("{:#08X}", word);
        }
    }

    pub(crate) fn push(&mut self, word: Word) {
        self.stack.push(word);
    }

    pub(crate) fn push_multi(&mut self, words: &[Word]) {
        self.stack.extend_from_slice(words);
    }

    pub(crate) fn push_zeros(&mut self, count: usize) {
        self.stack.extend(std::iter::repeat(0).take(count));
    }

    pub(crate) fn pop(&mut self) -> Word {
        self.stack.pop().unwrap()
    }

    pub(crate) fn pop_multi(&mut self, count: usize) {
        self.stack.truncate(self.stack.len() - count);
    }

    pub(crate) fn get_int(&self, index: usize) -> u32 {
        self.stack[index]
    }

    pub(crate) fn get_float(&self, index: usize) -> f32 {
        f32::from_bits(self.stack[index])
    }

    pub(crate) fn get_vec2_mut(&mut self, index: usize) -> &mut Vec2 {
        self.get_mut(index)
    }

    pub(crate) fn get_vec3_mut(&mut self, index: usize) -> &mut Vec3 {
        self.get_mut(index)
    }

    pub(crate) fn get_vec4_mut(&mut self, index: usize) -> &mut Vec4 {
        self.get_mut(index)
    }

    fn get<T>(&self, index: usize) -> &T {
        // Safely access to word aligned objects
        unsafe { &*(&self.stack[index] as *const Word as *const T) }
    }

    fn get_mut<T>(&mut self, index: usize) -> &mut T {
        // Safely access to word aligned objects
        unsafe { &mut *(&mut self.stack[index] as *mut Word as *mut T) }
    }
}