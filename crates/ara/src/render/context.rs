use derive_more::derive::{Deref, DerefMut};

use super::{Item, RenderContext, WeakItem};

pub trait ItemManager {
    /// create a new resource
    fn new_item<T: 'static>(&mut self, create: impl FnOnce(&mut ItemContext<T>) -> T) -> Item<T>;

    fn update_item<T: 'static, R>(
        &mut self,
        handle: &Item<T>,
        update: impl FnOnce(&mut T, &mut ItemContext<T>) -> R,
    ) -> anyhow::Result<R>;

    fn read_item<T: 'static, R>(
        &self,
        handle: &Item<T>,
        read: impl FnOnce(&T, &RenderContext) -> R,
    ) -> anyhow::Result<R>;
}

#[derive(Deref, DerefMut)]
pub struct ItemContext<'a, U: 'static> {
    #[deref]
    #[deref_mut]
    pub(crate) render_context: &'a mut RenderContext,
    weak_item: WeakItem<U>,
}

impl<'a, U: 'static> ItemContext<'a, U> {
    pub(crate) fn new(handle: WeakItem<U>, render_context: &'a mut RenderContext) -> Self {
        Self {
            render_context,
            weak_item: handle,
        }
    }
    pub fn item(&self) -> Item<U> {
        self.weak_item.upgrade().expect("Item released")
    }
}

impl<'a, U: 'static> ItemManager for ItemContext<'a, U> {
    fn new_item<T: 'static>(&mut self, create: impl FnOnce(&mut ItemContext<T>) -> T) -> Item<T> {
        self.render_context.new_item(create)
    }

    fn update_item<T: 'static, R>(
        &mut self,
        handle: &Item<T>,
        update: impl FnOnce(&mut T, &mut ItemContext<T>) -> R,
    ) -> anyhow::Result<R> {
        self.render_context.update_item(handle, update)
    }

    fn read_item<T: 'static, R>(
        &self,
        handle: &Item<T>,
        read: impl FnOnce(&T, &RenderContext) -> R,
    ) -> anyhow::Result<R> {
        self.render_context.read_item(handle, read)
    }
}
