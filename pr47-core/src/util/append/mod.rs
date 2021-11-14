pub trait Appendable {
    type Item;

    fn push_back(&mut self, item: Self::Item);
}

impl<T> Appendable for Vec<T> {
    type Item = T;

    fn push_back(&mut self, item: Self::Item) {
        self.push(item)
    }
}

#[cfg(feature = "compiler")]
use smallvec::{Array, SmallVec};

#[cfg(feature = "compiler")]
impl<A> Appendable for SmallVec<A>
    where A: Array
{
    type Item = A::Item;

    fn push_back(&mut self, item: Self::Item) {
        self.push(item)
    }
}
