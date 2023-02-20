pub(crate) const PAGE_SIZE: usize = 512;

#[derive(Default)]
pub(crate) struct PagedVector<T: Default + Copy> {
    pages: Vec<Option<Box<[T; PAGE_SIZE]>>>,
}

impl<T: Default + Copy> PagedVector<T> {

    pub(crate) fn new() -> Self {
        Self { pages: Vec::new() }
    }

    pub(crate) fn get(&self, index: usize) -> Option<&T> {
        let page = index / PAGE_SIZE;
        let offset = index % PAGE_SIZE;
        self.pages.get(page).and_then(|page| page.as_ref().map(|page| &page[offset]))
    }

    pub(crate) fn set(&mut self, index: usize, value: T) {
        let page = index / PAGE_SIZE;
        let offset = index % PAGE_SIZE;
        if page >= self.pages.len() {
            self.pages.resize_with(page + 1, || None)
        }
        if self.pages[page].is_none() {
            self.pages[page] = Some(Box::new([T::default(); PAGE_SIZE]));
        }
        self.pages[page].as_mut().unwrap()[offset] = value;
    }
}