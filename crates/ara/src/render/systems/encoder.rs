use crate::{
    render::{ItemContext, RenderContext, RenderRunner},
    Subscription,
};

use super::System;

pub struct EncoderSystem {
    encoder: Option<wgpu::CommandEncoder>,
    _sub: Option<Subscription>,
}

impl System for EncoderSystem {
    fn init(&mut self, _cx: &mut RenderContext) {}
}

impl EncoderSystem {
    pub fn new(cx: &mut ItemContext<Self>) -> Self {
        let s1 = cx.add_runner(RenderRunner::Start, |runner| {
            runner.update_system(|this: &mut Self, cx| {
                this.render_start(&cx.gpu);
            });
            Ok(())
        });

        let s2 = cx.add_runner(RenderRunner::PostRender, |runner| {
            runner.update_system(|this: &mut Self, cx| {
                this.post_render(&cx.gpu.queue);
            });
            Ok(())
        });

        Self {
            encoder: None,
            _sub: Some(Subscription::join(s1, s2)),
        }
    }

    fn render_start(&mut self, device: &wgpu::Device) {
        assert!(self.encoder.is_none(), "Encoder already exists");
        log::trace!("Starting new command encoder");
        self.encoder = Some(device.create_command_encoder(
            &(wgpu::CommandEncoderDescriptor {
                label: Some("ara::render::encoder::CommandEncoder"),
            }),
        ));
    }

    fn post_render(&mut self, queue: &wgpu::Queue) {
        if let Some(encoder) = self.encoder.take() {
            log::trace!("Finishing command encoder");
            queue.submit(Some(encoder.finish()));
        } else {
            log::warn!("No encoder to finish");
        }
    }

    pub fn with<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut wgpu::CommandEncoder) -> R,
    {
        if let Some(encoder) = self.encoder.as_mut() {
            f(encoder)
        } else {
            panic!("with_encoder should only be called while the encoder is active");
        }
    }
}
