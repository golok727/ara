pub mod arc_string;
pub mod canvas;
pub mod earcut;
pub mod gpu;
pub mod paint;
pub mod renderer;
pub mod text;

pub mod path;

pub use path::*;

pub use ara_math as math;

pub use canvas::Canvas;
pub use gpu::{ GpuContext, GpuContextCreateError };

pub use math::{ mat3, vec2, Corners, Mat3, Rect, Size, Vec2 };
pub use paint::color::{ Color, Rgba };
pub use paint::DrawList;
pub use paint::{
    circle,
    quad,
    AtlasKey,
    AtlasKeySource,
    AtlasTextureInfo,
    AtlasTextureInfoMap,
    Brush,
    Circle,
    FillStyle,
    LineCap,
    LineJoin,
    Quad,
    AraAtlas,
    StrokeStyle,
    Text,
    TextAlign,
    TextBaseline,
    TextureAtlas,
};

pub use canvas::{
    backend_target::BackendRenderTarget,
    offscreen_target::OffscreenRenderTarget,
    snapshot::{ CanvasSnapshot, CanvasSnapshotResult, CanvasSnapshotSource },
    target::RenderTarget,
};
pub use paint::{
    GpuTexture,
    GpuTextureView,
    GpuTextureViewDescriptor,
    Mesh,
    TextureAddressMode,
    TextureFilterMode,
    TextureFormat,
    TextureId,
    TextureKind,
    TextureOptions,
    PathBrush,
};

pub use renderer::{ Renderer2D, Renderer2DSpecs };

pub use text::{ Font, FontId, FontStyle, FontWeight, GlyphId, GlyphImage, TextSystem };

pub use ara_math::traits::*;
