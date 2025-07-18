#[derive(Debug, Clone)]
pub enum WebSurfaceTarget {
    OffscreenCanvas(web_sys::OffscreenCanvas),
    Canvas(web_sys::HtmlCanvasElement),
}

impl WebSurfaceTarget {
    pub fn try_as_canvas(&self) -> Option<&web_sys::HtmlCanvasElement> {
        match self {
            WebSurfaceTarget::OffscreenCanvas(_) => None,
            WebSurfaceTarget::Canvas(c) => Some(c),
        }
    }

    pub fn try_as_offscreen_canvas(&self) -> Option<&web_sys::OffscreenCanvas> {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => Some(c),
            WebSurfaceTarget::Canvas(_) => None,
        }
    }

    pub fn canvas(&self) -> &web_sys::HtmlCanvasElement {
        self.try_as_canvas().expect("Expected a canvas")
    }

    pub fn offscreen_canvas(&self) -> &web_sys::OffscreenCanvas {
        self.try_as_offscreen_canvas()
            .expect("Expected an offscreen canvas")
    }

    pub fn width(&self) -> u32 {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.width(),
            WebSurfaceTarget::Canvas(c) => c.width(),
        }
    }
    pub fn height(&self) -> u32 {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.height(),
            WebSurfaceTarget::Canvas(c) => c.height(),
        }
    }
    pub fn set_width(&self, width: u32) {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.set_width(width),
            WebSurfaceTarget::Canvas(c) => c.set_width(width),
        }
    }
    pub fn set_height(&self, height: u32) {
        match self {
            WebSurfaceTarget::OffscreenCanvas(c) => c.set_height(height),
            WebSurfaceTarget::Canvas(c) => c.set_height(height),
        }
    }
}

impl From<web_sys::HtmlCanvasElement> for WebSurfaceTarget {
    fn from(canvas: web_sys::HtmlCanvasElement) -> Self {
        Self::Canvas(canvas)
    }
}

impl From<web_sys::OffscreenCanvas> for WebSurfaceTarget {
    fn from(canvas: web_sys::OffscreenCanvas) -> Self {
        Self::OffscreenCanvas(canvas)
    }
}

impl From<WebSurfaceTarget> for wgpu::SurfaceTarget<'static> {
    fn from(surface: WebSurfaceTarget) -> Self {
        match surface {
            WebSurfaceTarget::OffscreenCanvas(c) => wgpu::SurfaceTarget::OffscreenCanvas(c),
            WebSurfaceTarget::Canvas(c) => wgpu::SurfaceTarget::Canvas(c),
        }
    }
}
