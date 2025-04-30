use crate::{ canvas::web_context::WebRenderingContext, RenderContextConfig, WebSurfaceTarget };

#[cfg(target_arch = "wasm32")]
impl super::Context {
    pub async fn new_web(
        surface_target: impl Into<WebSurfaceTarget>,
        render_target_config: &RenderContextConfig
    ) -> anyhow::Result<WebRenderingContext> {
        let target = WebRenderingContext::create(surface_target, render_target_config).await?;
        return Ok(target);
    }
}
