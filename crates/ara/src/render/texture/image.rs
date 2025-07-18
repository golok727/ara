use ara_math::Size;

use crate::render::ItemManager;

use super::RenderTexture;

pub struct Image {
    pub data: Option<Vec<u8>>,
}

impl Image {}

pub struct ImageHandle(pub usize);
// a image which can be used as a render target
pub struct RenderImage {
    pub handle: ImageHandle,
}

impl RenderTexture for RenderImage {
    fn resize(&self, _cx: &mut impl ItemManager, _physical_size: Size<u32>) {}
}
