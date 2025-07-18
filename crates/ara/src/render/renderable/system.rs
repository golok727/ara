use crate::{
    render::{systems::System, ItemContext, RenderRunner},
    Subscription,
};

// System responsible for directing renderables
pub struct RenderableSystem {
    _sub: Option<Subscription>,
}

impl RenderableSystem {
    pub fn new(cx: &mut ItemContext<Self>) -> Self {
        let sub = cx.add_runner(RenderRunner::PreRender, |runner| {
            runner.renderable.prepare(runner.cx);
            Ok(())
        });

        Self { _sub: Some(sub) }
    }
}

impl System for RenderableSystem {
    fn init(&mut self, _cx: &mut crate::render::RenderContext) {}
}
