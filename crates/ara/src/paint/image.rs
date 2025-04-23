use super::TextureKind;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct AtlasImageId(pub usize);

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct AtlasImage {
    pub(crate) image_id: AtlasImageId,
    pub(crate) texture_kind: TextureKind,
}

impl AtlasImage {}

impl AtlasImage {
    pub fn new(id: usize) -> Self {
        Self {
            image_id: AtlasImageId(id),
            texture_kind: TextureKind::Color,
        }
    }

    pub fn id(&self) -> &AtlasImageId {
        &self.image_id
    }

    pub fn texture_kind(&self) -> &TextureKind {
        &self.texture_kind
    }

    pub fn color(mut self) -> Self {
        self.texture_kind = TextureKind::Color;
        self
    }

    pub fn greyscale(mut self) -> Self {
        self.texture_kind = TextureKind::Mask;
        self
    }
}
