use wgpu::CommandEncoder;

use crate::{
    render::{
        renderable::Renderable,
        systems::{EncoderSystem, System},
        ItemContext, RenderContext, RenderRunner, RenderTargetView,
    },
    Color, Subscription,
};

use super::{backend::BackendRenderTargetAdapter, RenderTarget, RenderTargetAdapter};

#[derive(Debug, Clone, PartialEq)]
pub struct RenderTargetEntry {
    pub view: RenderTargetView,
}

pub struct RenderTargetSystem {
    adapter: RenderTargetAdapters,
    stack: Vec<RenderTargetEntry>,
    current: Option<RenderTargetEntry>,
    _sub: Option<Subscription>,
}

impl RenderTargetSystem {
    pub fn new(cx: &mut ItemContext<Self>) -> Self {
        let sub = cx
            .add_runner(RenderRunner::Start, |runner| {
                runner.cx.update_system(|this: &mut Self, _| {
                    this.push(RenderTargetEntry {
                        view: runner.view.clone(),
                    });
                });
                Ok(())
            })
            .join(cx.add_runner(RenderRunner::Render, |runner| {
                runner.cx.update_system(|this: &mut Self, cx| {
                    this.on_render(runner.renderable, runner.clear_color, cx)
                })
            }))
            .join(cx.add_runner(RenderRunner::Finish, |runner| {
                runner
                    .cx
                    .update_system(|this: &mut Self, cx| this.on_finish(cx))
            }));

        Self {
            stack: Default::default(),
            current: None,
            adapter: RenderTargetAdapters::default(),
            _sub: Some(sub),
        }
    }
}

impl System for RenderTargetSystem {
    fn init(&mut self, _cx: &mut crate::render::RenderContext) {}
}

impl RenderTargetSystem {
    pub fn push(&mut self, entry: RenderTargetEntry) {
        if let Some(current) = self.current.take() {
            self.stack.push(current); // save the current entry
        }
        self.current = Some(entry);
    }

    pub fn current_entry(&self) -> Option<&RenderTargetEntry> {
        self.current.as_ref()
    }

    pub fn current_target(&self) -> Option<&RenderTarget> {
        self.current.as_ref().map(|e| &e.view.target)
    }

    pub fn pop(&mut self) -> Option<RenderTargetEntry> {
        let current = self.current.take();
        self.current = self.stack.pop();
        current
    }

    fn on_finish(&mut self, cx: &mut RenderContext) -> anyhow::Result<()> {
        if let Some(current) = self.pop() {
            self.adapter.render_complete(&current.view.target, cx);
        }
        Ok(())
    }

    fn on_render(
        &mut self,
        renderable: &dyn Renderable,
        clear_color: Color,
        cx: &mut RenderContext,
    ) -> anyhow::Result<()> {
        let Some(entry) = &self.current else {
            // todo error handling
            return Ok(());
        };

        cx.update_system(|encoder: &mut EncoderSystem, cx| {
            encoder.with(|encoder| {
                let Some(mut pass) =
                    self.adapter
                        .begin_pass(&entry.view.target, clear_color, encoder, cx)
                else {
                    log::warn!("Error creating pass for target: {:?}", &entry.view.target);
                    return;
                };
                let viewport = entry.view.pixel_size;
                renderable.paint(&mut pass, viewport, cx);
            });
        });

        Ok(())
    }
}

#[derive(Default)]
struct RenderTargetAdapters {
    backend_adapter: BackendRenderTargetAdapter,
}

impl RenderTargetAdapters {
    fn begin_pass<'encoder>(
        &mut self,
        target: &RenderTarget,
        clear_color: Color,
        encoder: &'encoder mut CommandEncoder,
        cx: &mut RenderContext,
    ) -> Option<wgpu::RenderPass<'encoder>> {
        match target {
            RenderTarget::Backend(handle) => handle
                .update(cx, |target, cx| {
                    self.backend_adapter
                        .begin_pass(target, clear_color, encoder, cx)
                })
                .ok()
                .flatten(),
            RenderTarget::Noop => None,
        }
    }

    fn render_complete(&mut self, target: &RenderTarget, _cx: &mut RenderContext) {
        match target {
            RenderTarget::Backend(_) => {
                self.backend_adapter.render_complete();
            }
            RenderTarget::Noop => {
                // noop
            }
        }
    }
}
