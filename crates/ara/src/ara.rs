pub mod arc_string;
pub mod canvas;
pub mod earcut;
pub mod gpu;
pub mod paint;
pub mod renderer;
pub mod text;

pub mod slot;
pub use slot::*;

pub mod render;
pub mod scene;

pub mod path;

pub use path::*;

pub use ara_math as math;

pub use gpu::{Context, ContextSpecification, GpuContextCreateError};

pub use math::{mat3, vec2, Corners, Mat3, Rect, Size, Vec2};
pub use paint::color::{Color, Rgba};
pub use paint::DrawList;
pub use paint::{
    circle, quad, AraAtlas, AtlasKey, AtlasKeySource, AtlasTextureInfo, AtlasTextureInfoMap, Brush,
    Circle, FillStyle, LineCap, LineJoin, Quad, StrokeStyle, Text, TextAlign, TextBaseline,
    TextureAtlas,
};

pub use canvas::{
    backend_target::BackendRenderTarget,
    offscreen_target::OffscreenRenderTarget,
    render_context::{CanvasRenderTarget, CanvasRenderTargetDescriptor, MsaaSampleLevel},
    snapshot::{CanvasSnapshot, CanvasSnapshotResult, CanvasSnapshotSource},
    Canvas, CanvasConfig,
};
pub use paint::{
    GpuTexture, GpuTextureView, GpuTextureViewDescriptor, Mesh, PathBrush, TextureAddressMode,
    TextureFilterMode, TextureFormat, TextureId, TextureKind, TextureOptions,
};

pub use renderer::{Renderer2D, Renderer2DSpecs};

pub use text::{Font, FontId, FontStyle, FontWeight, GlyphId, GlyphImage, TextSystem};

pub use ara_math::traits::*;
