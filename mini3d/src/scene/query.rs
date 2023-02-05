use super::entity::Entity;

pub struct QueryIter<'a, Q: hecs::Query>(pub(crate) hecs::QueryIter<'a, Q>);

impl<'q, Q: hecs::Query> Iterator for QueryIter<'q, Q> {
    type Item = (Entity, hecs::QueryItem<'q, Q>);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(e, i)| (Entity(e), i))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

pub struct QueryIterBorrow<'a, Q: hecs::Query> {
    // pub(crate) query: hecs::QueryBorrow<'a, Q>,
    pub(crate) iter: hecs::QueryIter<'a, Q>,
}

impl<'a, Q: hecs::Query + 'a> QueryIterBorrow<'a, Q> {

    pub(crate) fn new(mut query: hecs::QueryBorrow<'a, Q>) -> Self {
        Self {
            // query,
            iter: query.iter(),
        }
    }
}

pub struct QueryView<'a, Q: hecs::Query>(pub(crate) hecs::View<'a, Q>);

impl<'a, Q: hecs::Query> QueryView<'a, Q> {
    pub fn get(&mut self, entity: Entity) -> Option<hecs::QueryItem<'a, Q>> {
        self.0.get_mut(entity.0)
    }
}