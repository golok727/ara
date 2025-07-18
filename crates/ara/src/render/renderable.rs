mod system;
pub use system::RenderableSystem;

use crate::scene::IntoSceneNode;
use crate::scene::RenderRoot;
use crate::scene::SceneNode;
use crate::scene::SceneNodeId;
use crate::scene::SceneNodeIdentifier;
use crate::Point;
use crate::Rect;

pub trait Renderable: DisplayObject {
    fn prepare(&self, render_context: &mut crate::render::RenderContext);
    fn paint<'encoder>(
        &self,
        pass: &mut wgpu::RenderPass<'encoder>,
        viewport: ara_math::Size<u32>,
        render_context: &mut crate::render::RenderContext,
    );
}

pub trait View {
    fn bounds(&self) -> Rect<f32>;
    fn contains_point(&self, point: Point) -> bool;
}

pub trait DisplayObject: View {
    fn get_position(&self) -> Point;
    fn get_scale(&self) -> Point;
    fn get_rotation(&self) -> f32;
    fn renderable(&self) -> bool;
    fn visible(&self) -> bool;
    fn alpha(&self) -> f32;
}

pub trait DisplayObjectMut: DisplayObject {
    fn set_position(&mut self, position: Point);
    fn set_scale(&mut self, scale: Point);
    fn set_rotation(&mut self, rotation: f32);
    fn set_visible(&mut self, visible: bool);
    fn set_alpha(&mut self, alpha: f32);
    fn set_renderable(&mut self, renderable: bool);
}

pub(crate) struct EmptyElement;

impl RenderRoot for EmptyElement {
    type Node = Self;

    fn node(&self) -> &Self::Node {
        self
    }
}

impl SceneNode for EmptyElement {
    fn prepare(&self, _render_context: &mut super::RenderContext) {}

    fn paint<'encoder>(
        &self,
        _pass: &mut wgpu::RenderPass<'encoder>,
        _viewport: ara_math::Size<u32>,
        _render_context: &mut super::RenderContext,
    ) {
    }
}

impl SceneNodeIdentifier for EmptyElement {
    fn id(&self) -> SceneNodeId {
        SceneNodeId::new()
    }
}

impl IntoSceneNode for EmptyElement {
    type Node = Self;

    fn into_scene_node(self) -> Self::Node {
        self
    }
}

impl View for EmptyElement {
    fn bounds(&self) -> Rect<f32> {
        Rect::default()
    }

    fn contains_point(&self, _point: Point) -> bool {
        false
    }
}

impl DisplayObject for EmptyElement {
    fn get_position(&self) -> Point {
        Point::default()
    }

    fn get_scale(&self) -> Point {
        Point::default()
    }

    fn get_rotation(&self) -> f32 {
        0.0
    }

    fn renderable(&self) -> bool {
        false
    }

    fn visible(&self) -> bool {
        false
    }

    fn alpha(&self) -> f32 {
        1.0
    }
}
