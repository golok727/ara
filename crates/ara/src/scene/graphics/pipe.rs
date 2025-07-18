use ara_math::{Rect, Size};

use crate::{
    paint::Vertex,
    render::{
        pipes::RenderPipe,
        systems::{GeometryBuilder, GeometrySystem, GlobalUniformSystem},
        Item, ItemContext, RenderCommand, RenderContext,
    },
    scene::{
        context::{BatchedGraphicsContextIter, BatchedGraphicsInstruction, GraphicsContext},
        path::GfxPathInstruction,
    },
    Circle, PathBrush, PathEventsIter, Quad,
};

use super::{GpuGraphicsContext, GraphicsContextSystem};

pub(crate) struct GraphicsPipe {
    pipeline: Option<wgpu::RenderPipeline>,
    #[allow(unused)]
    this: Item<Self>,
}

impl RenderPipe for GraphicsPipe {
    fn init(&mut self, cx: &mut RenderContext)
    where
        Self: Sized,
    {
        let device = &cx.gpu.device;

        let layout = cx.read_system(|sys: &GlobalUniformSystem, _| {
            device.create_pipeline_layout(
                &(wgpu::PipelineLayoutDescriptor {
                    label: Some("Global Uniform Bind Group Layout"),
                    bind_group_layouts: &[sys.get_bind_group_layout()],
                    push_constant_ranges: &[],
                }),
            )
        });

        let vbo_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x2, 2 => Float32x4],
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Graphics Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../../render/shaders/ara.wgsl").into()),
        });

        let blend = Some(wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::SrcAlpha,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent::OVER,
        });

        // todo move pipeline to pipeline system
        let pipeline = device.create_render_pipeline(
            &(wgpu::RenderPipelineDescriptor {
                label: Some("Graphics Pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs"),
                    buffers: &[vbo_layout],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some("fs_main"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        blend,
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleList,
                    strip_index_format: None,
                    front_face: wgpu::FrontFace::default(),
                    cull_mode: None,
                    unclipped_depth: false,
                    polygon_mode: wgpu::PolygonMode::Fill,
                    conservative: false,
                },
                depth_stencil: None,
                multisample: wgpu::MultisampleState {
                    count: 1,
                    mask: !0,
                    alpha_to_coverage_enabled: false,
                },
                multiview: None,
                cache: None,
            }),
        );

        self.pipeline.replace(pipeline);
    }
}

impl GraphicsPipe {
    pub fn new(cx: &mut ItemContext<Self>) -> Self {
        Self {
            this: cx.item(),
            pipeline: None,
        }
    }

    pub fn prepare(&self, cx: &mut RenderContext, context: &GraphicsContext) {
        if !context.dirty.get() {
            log::debug!(
                "Graphics context is not dirty, skipping rebuild for: {:?}",
                context.id()
            );
            return; // no need to rebuild reuse the old one
        }

        cx.update_system(|geometry_system: &mut GeometrySystem, cx| {
            cx.update_system(|graphics_context_system: &mut GraphicsContextSystem, _| {
                context.dirty.set(false);
                let gpu_context = graphics_context_system.get_or_init_cx(context, || {
                    GpuGraphicsContext::new(geometry_system.reserve())
                });

                let handle = gpu_context.geometry_handle;

                geometry_system.clear_data(handle);

                let batched_graphics_iter = BatchedGraphicsContextIter::new(context);

                let mut builder = GraphicsBuilder {
                    context,
                    batch: None,
                };

                gpu_context.clear();

                for batch in batched_graphics_iter {
                    let clip_rect = batch.clip_rect.clone();

                    builder.set_batch(batch);

                    let slice = geometry_system.append_data(handle, &mut builder);

                    if !slice.is_empty() {
                        gpu_context.add_command(RenderCommand::SetScissor { rect: clip_rect });
                        gpu_context.add_command(RenderCommand::draw_indexed(handle, slice));
                    }
                }

                geometry_system.sync(handle);
            });
        });
    }

    pub fn execute(
        &self,
        pass: &mut wgpu::RenderPass,
        viewport: Size<u32>,
        cx: &mut RenderContext,
        context: &GraphicsContext,
    ) {
        let Some(pipeline) = self.pipeline.as_ref() else {
            log::warn!("GraphicsPipe not initialized");
            return;
        };

        cx.read_system(|graphics_context_system: &GraphicsContextSystem, cx| {
            let Some(gpu_context) = graphics_context_system.get_cx(context) else {
                log::debug!("Empty context skipping: {:?}", context.id());
                return;
            };

            pass.set_pipeline(pipeline);
            cx.read_system(|sys: &GlobalUniformSystem, _| {
                pass.set_bind_group(0, sys.get_bind_group(), &[]);
            });

            cx.read_system(|geometry_system: &GeometrySystem, _| {
                /* End Read geometry system */
                for command in &gpu_context.commands {
                    match command {
                        RenderCommand::SetScissor { rect } => {
                            let scissor = ScissorRect::new(rect, &viewport);
                            pass.set_scissor_rect(
                                scissor.x,
                                scissor.y,
                                scissor.width,
                                scissor.height,
                            );
                        }
                        RenderCommand::DrawIndexed {
                            geometry_handle,
                            render_buffer_slice,
                        } => {
                            if let Some(buffer) =
                                geometry_system.get(*geometry_handle, render_buffer_slice)
                            {
                                pass.set_vertex_buffer(0, buffer.vertex_buffer);

                                pass.set_index_buffer(
                                    buffer.index_buffer,
                                    wgpu::IndexFormat::Uint32,
                                );

                                pass.draw_indexed(0..buffer.index_count, 0, 0..1);
                            }
                        }
                    }
                }
                /* END Read Geometry System */
            });

            pass.set_scissor_rect(0, 0, viewport.width, viewport.height);
        });
    }
}

struct GraphicsBuilder<'a> {
    batch: Option<BatchedGraphicsInstruction<'a>>,
    context: &'a GraphicsContext,
}

impl<'a> GraphicsBuilder<'a> {
    fn set_batch(&mut self, batch: BatchedGraphicsInstruction<'a>) {
        self.batch = Some(batch);
    }
}

impl GeometryBuilder for GraphicsBuilder<'_> {
    fn build(&mut self, drawlist: &mut crate::DrawList) {
        let batch = self.batch.as_ref().expect("Expected a batch");

        // todo - remove brush and directly use the fill and stroke in drawlist
        let mut brush = PathBrush::default();

        drawlist.feathering(2.0);
        brush.default.antialias = true;

        let transform = *batch.transform;

        if let Some(fill) = batch.fill {
            brush.default.fill_style = *fill;
        }

        if let Some(stroke) = batch.stroke {
            brush.default.stroke_style = *stroke;
        }

        for instruction in batch.path_instructions {
            match instruction {
                GfxPathInstruction::Rect { bounds } => {
                    drawlist.add_quad(
                        &Quad::default().rect(bounds.clone()),
                        &brush.default,
                        false,
                        Some(transform),
                    );
                }
                GfxPathInstruction::RoundRect { bounds, corners } => {
                    drawlist.add_quad(
                        &Quad::default()
                            .rect(bounds.clone())
                            .corners(corners.clone()),
                        &brush.default,
                        false,
                        Some(transform),
                    );
                }
                GfxPathInstruction::Circle { center, radius } => {
                    drawlist.add_circle(
                        &Circle::default().pos(center.x, center.y).radius(*radius),
                        &brush.default,
                        false,
                        Some(transform),
                    );
                }

                GfxPathInstruction::Path { points, verbs } => {
                    let points = &self.context.path.builder.points[points.clone()];

                    let verbs = &self.context.path.builder.verbs[verbs.clone()];

                    let iter = PathEventsIter::new(points, verbs);

                    drawlist.add_path(iter, &brush, Some(transform));
                }
            }
        }
    }
}

struct ScissorRect {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl ScissorRect {
    fn new(clip_rect: &Rect<f32>, screen_size: &Size<u32>) -> Self {
        let clip_min = clip_rect.min().round().map(|v| v as u32);
        let clip_max = clip_rect.max().round().map(|v| v as u32);

        let clip_min_x = clip_min.x.clamp(0, screen_size.width);
        let clip_min_y = clip_min.y.clamp(0, screen_size.height);
        let clip_max_x = clip_max.x.clamp(clip_min_x, screen_size.width);
        let clip_max_y = clip_max.y.clamp(clip_min_y, screen_size.height);

        Self {
            x: clip_min_x,
            y: clip_min_y,
            width: clip_max_x - clip_min_x,
            height: clip_max_y - clip_min_y,
        }
    }
}
