use std::cell::Cell;

use ara_math::Size;
use wgpu::util::DeviceExt;

use crate::{
    render::{ItemContext, RenderRunner},
    Subscription,
};

use super::System;

pub struct GlobalUniformSystem {
    data: GlobalUniformData,
    queue: wgpu::Queue,
    buffer: GlobalUniformsBuffer,
    _sub: Option<Subscription>,
}

impl GlobalUniformSystem {
    pub fn new(cx: &mut ItemContext<Self>) -> Self {
        let data = GlobalUniformData::default();
        let buffer = GlobalUniformsBuffer::new(cx.gpu(), data);

        let _sub = cx.add_runner(RenderRunner::PreRender, move |runner| {
            let screen = runner.view.screen_size;
            runner.update_system(|this: &mut Self, _| {
                this.prepare(screen.map(|v| v as f32));
            });
            Ok(())
        });

        Self {
            data,
            queue: cx.gpu.queue.clone(),
            buffer,
            _sub: Some(_sub),
        }
    }
}

impl System for GlobalUniformSystem {
    fn init(&mut self, _: &mut crate::render::RenderContext) {}
}

impl GlobalUniformSystem {
    fn map<F>(&mut self, f: F)
    where
        F: FnOnce(&mut GlobalUniformData),
    {
        f(&mut self.data);
        self.buffer.set_data(self.data);
        self.buffer.sync(&self.queue);
    }

    fn prepare(&mut self, screen: Size<f32>) {
        self.map(|data| {
            data.set_size(screen);
        });
    }

    pub fn get_bind_group_layout(&self) -> &wgpu::BindGroupLayout {
        &self.buffer.bing_group_layout
    }

    pub fn get_bind_group(&self) -> &wgpu::BindGroup {
        &self.buffer.bind_group
    }
}

#[derive(Debug, Clone, Copy, bytemuck::Zeroable, bytemuck::Pod, PartialEq)]
#[repr(C)]
pub struct GlobalUniformData {
    screen_size: [f32; 2],
    _pad: [f32; 2], // for webgl
}

impl GlobalUniformData {
    pub fn set_size(&mut self, size: Size<f32>) {
        self.screen_size = [size.width, size.height];
    }
}

impl Default for GlobalUniformData {
    fn default() -> Self {
        Self {
            screen_size: [1.0, 1.0],
            _pad: Default::default(),
        }
    }
}

#[derive(Debug)]
pub struct GlobalUniformsBuffer {
    pub data: GlobalUniformData,
    pub gpu_buffer: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    pub bing_group_layout: wgpu::BindGroupLayout,
    dirty: Cell<bool>,
}

impl GlobalUniformsBuffer {
    pub fn new(device: &wgpu::Device, data: GlobalUniformData) -> Self {
        let gpu_buffer = device.create_buffer_init(
            &(wgpu::util::BufferInitDescriptor {
                label: Some("Global uniform buffer"),
                contents: bytemuck::cast_slice(&[data]),
                usage: wgpu::BufferUsages::UNIFORM
                    | wgpu::BufferUsages::COPY_SRC
                    | wgpu::BufferUsages::COPY_DST,
            }),
        );

        let layout = device.create_bind_group_layout(
            &(wgpu::BindGroupLayoutDescriptor {
                label: Some("Global uniform bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            }),
        );

        let bind_group = device.create_bind_group(
            &(wgpu::BindGroupDescriptor {
                label: Some("Global uniform bind group"),
                layout: &layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: gpu_buffer.as_entire_binding(),
                }],
            }),
        );

        Self {
            data,
            gpu_buffer,
            bind_group,
            bing_group_layout: layout,
            dirty: Cell::new(false),
        }
    }

    pub fn set_data(&mut self, data: GlobalUniformData) {
        if self.data == data {
            return;
        }
        self.data = data;
        self.dirty.set(true);
    }

    #[allow(unused)]
    pub fn map(&mut self, f: impl FnOnce(&mut GlobalUniformData)) {
        f(&mut self.data);
        self.dirty.set(true);
    }

    pub fn sync(&self, queue: &wgpu::Queue) {
        if !self.dirty.get() {
            return;
        }

        queue.write_buffer(&self.gpu_buffer, 0, bytemuck::cast_slice(&[self.data]));

        self.dirty.set(false);
    }
}
