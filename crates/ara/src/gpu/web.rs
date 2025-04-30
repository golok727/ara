use crate::{canvas::web::WebRenderContext, RenderContextConfig, WebSurfaceTarget};

#[cfg(target_arch = "wasm32")]
impl super::Context {
    pub async fn new_web(
        canvas: impl Into<WebSurfaceTarget>,
        render_target_config: &RenderContextConfig,
    ) -> anyhow::Result<WebRenderContext> {
        let target = WebRenderContext::create(canvas, render_target_config).await?;
        return Ok(target);
    }
}
