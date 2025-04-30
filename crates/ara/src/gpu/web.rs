use crate::{
    canvas::web_context::WebRenderingContext,
    CanvasRenderingContextConfig,
    WebSurfaceTarget,
};

#[cfg(target_arch = "wasm32")]
impl super::Context {
    pub async fn new_web(
        surface_target: impl Into<WebSurfaceTarget>,
        render_target_config: &CanvasRenderingContextConfig
    ) -> anyhow::Result<WebRenderingContext> {
        let target = WebRenderingContext::create(surface_target, render_target_config).await?;
        return Ok(target);
    }
}
