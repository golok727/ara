use derive_more::derive::{Deref, DerefMut};

use crate::{Color, Slot};

use super::{
    renderable::Renderable, ItemContext, ItemManager, RenderContext, RenderRunner, RenderTargetView,
};

#[derive(Deref, DerefMut)]
pub struct RenderExecContext<'a> {
    pub view: &'a RenderTargetView,
    pub kind: RenderRunner,
    pub clear_color: Color,
    pub renderable: &'a dyn Renderable,
    #[deref]
    #[deref_mut]
    pub cx: &'a mut RenderContext,
}

pub type RenderRunnerFn = Box<dyn (Fn(&mut RenderExecContext) -> anyhow::Result<()>) + 'static>;

#[derive(Default)]
pub struct RenderRunners {
    pub start: Slot<RenderRunnerFn>,
    pub prerender: Slot<RenderRunnerFn>,
    pub render: Slot<RenderRunnerFn>,
    pub postrender: Slot<RenderRunnerFn>,
    pub finish: Slot<RenderRunnerFn>,
}

impl<'a> ItemManager for RenderExecContext<'a> {
    fn new_item<T: 'static>(
        &mut self,
        create: impl FnOnce(&mut ItemContext<T>) -> T,
    ) -> super::Item<T> {
        self.cx.new_item(create)
    }

    fn update_item<T: 'static, R>(
        &mut self,
        handle: &super::Item<T>,
        update: impl FnOnce(&mut T, &mut ItemContext<T>) -> R,
    ) -> anyhow::Result<R> {
        self.cx.update_item(handle, update)
    }

    fn read_item<T: 'static, R>(
        &self,
        handle: &super::Item<T>,
        read: impl FnOnce(&T, &RenderContext) -> R,
    ) -> anyhow::Result<R> {
        self.cx.read_item(handle, read)
    }
}
