use std::{ borrow::Cow, sync::Arc };

use crate::{
    circle,
    paint::{
        AtlasKey,
        Brush,
        GpuTextureView,
        GraphicsInstruction,
        GraphicsInstructionBatcher,
        PathBrush,
        Primitive,
        AraAtlas,
        AraAtlasTextureInfoMap,
        TextureKind,
    },
    path::Path,
    quad,
    renderer::Renderable,
    AtlasTextureInfo,
    Color,
    DrawList,
    GlyphImage,
    IsZero,
    Rect,
    Renderer2D,
    Size,
    Text,
    TextSystem,
    TextureId,
    TextureOptions,
};
use ahash::HashSet;
use anyhow::Result;
use cosmic_text::{ Attrs, Buffer, Metrics, Shaping };
use ara_math::{ Corners, Mat3, Vec2 };
use target::{ RenderTarget, RenderTargetConfig };
use wgpu::FilterMode;

pub mod backend_target;
pub mod builder;
pub mod offscreen_target;
pub mod render_list;
pub mod snapshot;
pub mod target;

use render_list::RenderList;

pub use builder::CanvasBuilder;

#[derive(Debug, Clone, PartialEq)]
pub struct CanvasState {
    pub transform: Mat3,
    pub clip_rect: Rect<f32>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            transform: Mat3::identity(),
            clip_rect: Rect::EVERYTHING,
        }
    }
}

pub struct Canvas {
    // TODO
    // - pub(crate)
    // - allow rendering in another thread
    pub renderer: Renderer2D,

    pub(crate) surface_config: RenderTargetConfig,

    list: RenderList,
    texture_atlas: Arc<AraAtlas>,
    text_system: Arc<TextSystem>,

    atlas_info_map: AraAtlasTextureInfoMap,

    state_stack: Vec<CanvasState>,
    current_state: CanvasState,

    cached_renderables: Vec<Renderable>,

    white_texture_uv: Vec2<f32>,

    clear_color: Color,
}

impl Canvas {
    // TODO make it configurable
    const AA_SIZE: f32 = 3.0;

    pub(super) fn new(
        surface_config: RenderTargetConfig,
        renderer: Renderer2D,
        texture_atlas: Arc<AraAtlas>,
        text_system: Arc<TextSystem>
    ) -> Self {
        // hoping it wont change
        let white_texture_uv = texture_atlas
            .get_texture_info(&AtlasKey::WhiteTexture)
            .map(|info| info.uv_to_atlas_space(0.0, 0.0))
            .expect("unable to get white_texture_uv");

        Canvas {
            renderer,

            texture_atlas,
            text_system,

            atlas_info_map: Default::default(),

            state_stack: Default::default(),

            clear_color: Color::WHITE,
            current_state: CanvasState::default(),

            surface_config,

            white_texture_uv,

            list: Default::default(),
            cached_renderables: Default::default(),
        }
    }

    pub fn create() -> CanvasBuilder {
        CanvasBuilder::default()
    }

    pub fn screen(&self) -> Size<u32> {
        Size::new(self.surface_config.width, self.surface_config.height)
    }

    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    pub fn atlas(&self) -> &Arc<AraAtlas> {
        &self.texture_atlas
    }

    pub fn text_system(&self) -> &Arc<TextSystem> {
        &self.text_system
    }

    pub fn get_clip_rect(&self) -> Rect<f32> {
        self.current_state.clip_rect.clone()
    }

    pub fn save(&mut self) {
        self.stage_changes();
        self.state_stack.push(self.current_state.clone());
    }

    pub fn clear_color(&mut self, clear_color: Color) {
        self.clear_color = clear_color;
    }

    pub fn restore(&mut self) {
        if let Some(state) = self.state_stack.pop() {
            self.stage_changes();
            self.current_state = state;
        }
    }

    pub fn reset(&mut self) {
        self.stage_changes();

        self.clear_color = Color::WHITE;
        self.current_state = CanvasState {
            transform: Mat3::identity(),
            clip_rect: Rect::EVERYTHING,
        };

        self.state_stack.clear();
    }

    pub fn clip(&mut self, rect: &Rect<f32>) {
        self.stage_changes();
        self.current_state.clip_rect = self.current_state.clip_rect.intersect(rect);
    }

    pub fn translate(&mut self, dx: f32, dy: f32) {
        self.stage_changes();
        self.current_state.transform.translate(dx, dy);
    }

    pub fn scale(&mut self, sx: f32, sy: f32) {
        self.stage_changes();
        self.current_state.transform.scale(sx, sy);
    }

    pub fn rotate(&mut self, angle_rad: f32) {
        self.stage_changes();
        self.current_state.transform.rotate(angle_rad);
    }

    pub fn clear(&mut self) {
        self.list.clear();
        self.cached_renderables.clear();
    }

    #[inline]
    pub fn stage_changes(&mut self) {
        self.list.stage_changes(self.current_state.clone());
    }

    #[inline]
    pub fn draw_primitive(&mut self, prim: impl Into<Primitive>, brush: Brush) {
        self.list.add(GraphicsInstruction::brush(prim, brush.clone()));
    }

    pub fn draw_path(&mut self, path: impl Into<Path>, brush: impl Into<PathBrush>) {
        self.draw_primitive(
            Primitive::Path {
                path: path.into(),
                brush: brush.into(),
            },
            // FIXME: This is a workaround
            Brush::filled(Color::WHITE)
        );
    }

    pub fn draw_rect(&mut self, rect: &Rect<f32>, brush: Brush) {
        self.draw_primitive(quad().rect(rect.clone()), brush);
    }

    pub fn draw_round_rect(&mut self, rect: &Rect<f32>, corners: &Corners<f32>, brush: Brush) {
        self.draw_primitive(quad().rect(rect.clone()).corners(corners.clone()), brush);
    }

    pub fn draw_image(&mut self, rect: &Rect<f32>, texture_id: &TextureId) {
        self.list.add(GraphicsInstruction::textured(quad().rect(rect.clone()), texture_id.clone()));
    }

    pub fn draw_image_rounded(
        &mut self,
        rect: &Rect<f32>,
        corners: &Corners<f32>,
        texture_id: &TextureId
    ) {
        self.list.add(
            GraphicsInstruction::textured(
                quad().rect(rect.clone()).corners(corners.clone()),
                texture_id.clone()
            )
        );
    }

    pub fn draw_circle(&mut self, cx: f32, cy: f32, radius: f32, brush: Brush) {
        self.draw_primitive(circle().pos(cx, cy).radius(radius), brush);
    }

    pub fn fill_text(&mut self, text: &Text, fill_color: Color) {
        self.stage_changes();
        self.text_system.write(|state| {
            let line_height_em = 1.4;
            let metrics = Metrics::new(text.size, text.size * line_height_em);
            let mut buffer = Buffer::new(&mut state.font_system, metrics);
            buffer.set_size(
                &mut state.font_system,
                Some(self.surface_config.width as f32),
                Some(self.surface_config.height as f32)
            );

            let attrs = Attrs::new();
            attrs.style(text.font.style.into());
            attrs.weight(text.font.weight.into());
            attrs.family(cosmic_text::Family::Name(&text.font.family));

            buffer.set_text(&mut state.font_system, &text.text, attrs, Shaping::Advanced);

            buffer.shape_until_scroll(&mut state.font_system, false);
            // begin run
            for run in buffer.layout_runs() {
                let line_y = run.line_y;

                // begin glyphs
                for glyph in run.glyphs.iter() {
                    let scale = 1.0;
                    let physical_glyph = glyph.physical((text.pos.x, text.pos.y), scale);
                    let image = state.swash_cache.get_image(
                        &mut state.font_system,
                        physical_glyph.cache_key
                    );

                    if let Some(image) = image {
                        let kind = match image.content {
                            cosmic_text::SwashContent::Color => TextureKind::Color,
                            cosmic_text::SwashContent::Mask => TextureKind::Mask,
                            // we don't support it for now
                            cosmic_text::SwashContent::SubpixelMask => TextureKind::Mask,
                        };

                        let glyph_key = AtlasKey::from(GlyphImage {
                            key: physical_glyph.cache_key,
                            is_emoji: kind.is_color(),
                        });

                        let size = Size::new(
                            image.placement.width as i32,
                            image.placement.height as i32
                        );

                        if size.is_zero() {
                            continue;
                        }

                        self.texture_atlas.get_or_insert(&glyph_key, || (
                            size,
                            Cow::Borrowed(&image.data),
                        ));

                        self.renderer.set_texture_from_atlas(
                            &self.texture_atlas,
                            &glyph_key,
                            &TextureOptions::default()
                                .min_filter(FilterMode::Nearest)
                                .mag_filter(FilterMode::Nearest)
                        );

                        let x = physical_glyph.x + image.placement.left;
                        let y = (line_y as i32) + physical_glyph.y - image.placement.top;

                        let color = if kind.is_color() {
                            let mut c = Color::WHITE;
                            c.a = fill_color.a;
                            c
                        } else {
                            fill_color
                        };

                        self.list.add(
                            GraphicsInstruction::textured_brush(
                                quad().rect(
                                    Rect::from_origin_size(
                                        (x as f32, y as f32).into(),
                                        size.map(|v| *v as f32)
                                    )
                                ),
                                TextureId::AtlasKey(glyph_key),
                                Brush::filled(color)
                            )
                        );
                    }
                }
                // end glyphs
            }
            // end run
        });
        self.stage_changes();
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        let width = new_width.max(1);
        let height = new_height.max(1);

        self.renderer.resize(width, height);
        self.surface_config.width = width;
        self.surface_config.height = height;
    }

    pub fn render<Surface, Output>(&mut self, surface: &mut Surface) -> Result<Output>
        where Surface: RenderTarget<PaintOutput = Output>
    {
        if surface.get_config() != self.surface_config {
            log::trace!("{}: surface.configure() ran", Surface::LABEL);
            surface.configure(self.renderer.gpu(), &self.surface_config);
        }

        surface.paint(self)
    }

    pub(crate) fn render_to_texture(
        &mut self,
        view: &GpuTextureView,
        resolve_target: Option<&wgpu::TextureView>
    ) {
        self.prepare_for_render();

        let mut encoder = self.renderer.create_command_encoder();

        {
            let mut pass = encoder.begin_render_pass(
                &(wgpu::RenderPassDescriptor {
                    label: Some("RenderTarget Pass"),
                    color_attachments: &[
                        Some(wgpu::RenderPassColorAttachment {
                            view,
                            resolve_target,
                            ops: wgpu::Operations {
                                load: wgpu::LoadOp::Clear(self.clear_color.into()),
                                store: wgpu::StoreOp::Store,
                            },
                        }),
                    ],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                })
            );

            self.renderer.prepare(&self.cached_renderables);
            self.renderer.render(&mut pass, &self.cached_renderables);
        }

        self.renderer.gpu().queue.submit(std::iter::once(encoder.finish()));
    }

    fn get_required_atlas_keys(&self) -> HashSet<AtlasKey> {
        self.list
            .into_iter()
            .flat_map(|staged| staged.instructions.iter())
            .filter_map(|instruction| {
                if let TextureId::AtlasKey(key) = &instruction.texture_id {
                    Some(key.clone())
                } else {
                    None
                }
            })
            .collect::<_>()
    }

    fn prepare_for_render(&mut self) {
        // stage the any remaining changes
        self.stage_changes();

        // prepare atlas texture infos
        let atlas_keys = self.get_required_atlas_keys();

        for key in atlas_keys {
            if self.atlas_info_map.contains_key(&key) {
                continue;
            }
            let info = self.texture_atlas.get_texture_info(&key);

            if let Some(info) = info {
                self.atlas_info_map.insert(key.clone(), info);
            } else {
                log::error!("Cannot find info for key in atlas : {:#?}", key);
            }
        }

        let get_renderer_texture = |texture_id: &TextureId| {
            match texture_id {
                TextureId::AtlasKey(key) =>
                    self.atlas_info_map.get(key).map(|info| TextureId::Atlas(info.tile.texture)),
                _ => None, // the batcher will use the instruction.texture
            }
        };

        let mut drawlist = DrawList::default();
        drawlist.feathering(Self::AA_SIZE);

        // TODO batch ops in stages too
        for staged in &self.list {
            let batcher = GraphicsInstructionBatcher::new(
                staged.instructions,
                get_renderer_texture
            );

            for batch in batcher {
                let render_texture = batch.renderer_texture.clone();
                if
                    let Some(renderable) = self.build_renderable(
                        &mut drawlist,
                        batch,
                        render_texture,
                        staged.state
                    )
                {
                    self.cached_renderables.push(renderable);
                }
            }
        }
    }

    fn build_renderable<'a>(
        &self,
        drawlist: &mut DrawList,
        instructions: impl Iterator<Item = &'a GraphicsInstruction>,
        render_texture: TextureId,
        canvas_state: &CanvasState
    ) -> Option<Renderable> {
        for instruction in instructions {
            let primitive = &instruction.primitive;
            let brush = &instruction.brush;

            if instruction.nothing_to_draw() {
                return None;
            }

            let tex_id = instruction.texture_id.clone();
            let is_white_texture = tex_id == TextureId::WHITE_TEXTURE;

            let info: Option<&AtlasTextureInfo> = if let TextureId::AtlasKey(key) = &tex_id {
                self.atlas_info_map.get(key)
            } else {
                None
            };

            let build = |drawlist: &mut DrawList| {
                drawlist.add_primitive(
                    primitive,
                    brush,
                    !is_white_texture,
                    Some(canvas_state.transform)
                )
            };

            if info.is_none() {
                build(drawlist);
            } else {
                drawlist.capture(build).map(|vertex| {
                    if let Some(info) = info {
                        if is_white_texture {
                            vertex.uv = self.white_texture_uv.into();
                        } else {
                            vertex.uv = info.uv_to_atlas_space(vertex.uv[0], vertex.uv[1]).into();
                        }
                    }
                });
            }
        }

        let mut mesh = drawlist.build();
        if mesh.is_empty() {
            return None;
        }

        mesh.texture = render_texture.clone();

        Some(Renderable {
            clip_rect: canvas_state.clip_rect.clone(),
            mesh,
        })
    }
}
