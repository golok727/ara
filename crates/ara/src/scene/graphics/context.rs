use std::cell::Cell;
use std::sync::atomic::AtomicUsize;

use crate::{Color, PathEvent};
use ara_math::{Corners, Mat3, Point};

use crate::{math::Rect, StrokeStyle};

use crate::{FillStyle, LineCap, LineJoin};

use super::path::{GfxPathEntry, GfxPathInstruction, GraphicsPath};

#[derive(Debug, Clone, Eq, Hash, PartialEq)]
pub enum GraphicsInstructionKind {
    Fill {
        fill_style: FillStyle,
        path: GfxPathEntry,
    },
    Stroke {
        stroke_style: StrokeStyle,
        path: GfxPathEntry,
    },
    // Texture {
    //     id: TextureId,
    // },
}

impl GraphicsInstructionKind {
    pub fn path(&self) -> GfxPathEntry {
        match self {
            GraphicsInstructionKind::Fill { path, .. } => *path,
            GraphicsInstructionKind::Stroke { path, .. } => *path,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GraphicsInstruction {
    pub kind: GraphicsInstructionKind,
    pub transform: Mat3,
    pub clip_rect: Rect<f32>,
}

#[derive(Clone)]
struct State {
    pub transform: Mat3,
    pub clip_rect: Rect<f32>,
    pub line_width: u32,
    pub line_join: LineJoin,
    pub line_cap: LineCap,
}

impl Default for State {
    fn default() -> Self {
        Self {
            transform: Default::default(),
            clip_rect: Rect::EVERYTHING,
            line_width: 2,
            line_join: LineJoin::Miter,
            line_cap: LineCap::Butt,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GraphicsContextId(pub(crate) usize);

impl GraphicsContextId {
    pub(crate) fn new() -> Self {
        static NEXT_ID: AtomicUsize = AtomicUsize::new(0);
        let id = NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Self(id)
    }
}

pub struct GraphicsContext {
    pub(crate) id: GraphicsContextId,
    pub(crate) path: GraphicsPath,
    pub(crate) instructions: Vec<GraphicsInstruction>,

    pub(crate) dirty: Cell<bool>,

    stack: Vec<State>,
    cur_state: State,
}

unsafe impl Send for GraphicsContext {}

unsafe impl Sync for GraphicsContext {}

impl Default for GraphicsContext {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for GraphicsContext {
    fn clone(&self) -> Self {
        Self {
            id: GraphicsContextId::new(),
            dirty: self.dirty.clone(),
            path: self.path.clone(),
            instructions: self.instructions.clone(),
            stack: self.stack.clone(),
            cur_state: self.cur_state.clone(),
        }
    }
}

impl GraphicsContext {
    pub fn id(&self) -> GraphicsContextId {
        self.id
    }

    pub(crate) fn new() -> Self {
        let stack = Vec::new();
        let cur_state = State::default();
        let path = GraphicsPath::default();
        let instructions = Vec::new();

        Self {
            id: GraphicsContextId::new(),
            dirty: Cell::new(false),
            stack,
            cur_state,
            path,
            instructions,
        }
    }
}

impl GraphicsContext {
    pub fn clear(&mut self) -> &mut Self {
        self.dirty.set(true);
        self.path.clear();
        self.instructions.clear();
        self
    }

    pub fn save(&mut self) -> &mut Self {
        self.stack.push(self.cur_state.clone());
        self
    }

    pub fn restore(&mut self) -> &mut Self {
        if let Some(state) = self.stack.pop() {
            self.cur_state = state;
        }
        self
    }

    pub fn set_clip(&mut self, rect: Rect<f32>) -> &mut Self {
        self.cur_state.clip_rect = self.cur_state.clip_rect.intersect(&rect);
        self
    }

    pub fn get_clip(&self) -> Rect<f32> {
        self.cur_state.clip_rect.clone()
    }

    pub fn reset_clip(&mut self) -> &mut Self {
        self.cur_state.clip_rect = Rect::EVERYTHING;
        self
    }

    /// Reset the current state to default values
    pub fn reset(&mut self) -> &mut Self {
        self.cur_state = State::default();
        self
    }

    pub fn reset_transform(&mut self) -> &mut Self {
        self.cur_state.transform = Mat3::IDENTITY;
        self
    }

    pub fn set_line_width(&mut self, line_width: u32) -> &mut Self {
        self.cur_state.line_width = line_width;
        self
    }

    pub fn get_line_width(&self) -> u32 {
        self.cur_state.line_width
    }

    pub fn set_line_join(&mut self, line_join: LineJoin) -> &mut Self {
        self.cur_state.line_join = line_join;
        self
    }

    pub fn get_line_join(&self) -> LineJoin {
        self.cur_state.line_join
    }

    pub fn set_line_cap(&mut self, line_cap: LineCap) -> &mut Self {
        self.cur_state.line_cap = line_cap;
        self
    }

    pub fn get_line_cap(&self) -> LineCap {
        self.cur_state.line_cap
    }

    pub fn translate(&mut self, dx: f32, dy: f32) -> &mut Self {
        self.cur_state.transform.translate(dx, dy);
        self
    }

    pub fn scale(&mut self, sx: f32, sy: f32) -> &mut Self {
        self.cur_state.transform.scale(sx, sy);
        self
    }

    pub fn rotate(&mut self, angle_rad: f32) -> &mut Self {
        self.cur_state.transform.rotate(angle_rad);
        self
    }

    pub fn path<T>(&mut self, path: T) -> &mut Self
    where
        T: IntoIterator<Item = PathEvent>,
    {
        self.path.path2d(path);
        self
    }

    pub fn rect(&mut self, rect: Rect<f32>) -> &mut Self {
        self.path.rect(rect);
        self
    }

    pub fn round_rect(&mut self, rect: Rect<f32>, radii: Corners<f32>) -> &mut Self {
        self.path.round_rect(rect, radii);
        self
    }

    pub fn circle(&mut self, center: Point, radius: f32) -> &mut Self {
        self.path.circle(center, radius);
        self
    }

    pub fn fill(&mut self, color: impl Into<Color>) -> &mut Self {
        if let Some(path) = self.path.push() {
            self.dirty.set(true);
            let state = &self.cur_state;

            let kind = GraphicsInstructionKind::Fill {
                fill_style: FillStyle::default().color(color.into()),
                path,
            };

            let ins = GraphicsInstruction {
                kind,
                transform: state.transform,
                clip_rect: state.clip_rect.clone(),
            };

            if Some(&ins) != self.instructions.last() {
                self.instructions.push(ins);
            }
        }
        self
    }

    pub fn stroke(&mut self, color: impl Into<Color>) -> &mut Self {
        if let Some(path) = self.path.push() {
            self.dirty.set(true);
            let state = &self.cur_state;

            let kind = GraphicsInstructionKind::Stroke {
                stroke_style: StrokeStyle {
                    color: color.into(),
                    line_width: state.line_width,
                    line_join: state.line_join,
                    line_cap: state.line_cap,
                    ..Default::default()
                },
                path,
            };
            let ins = GraphicsInstruction {
                kind,
                transform: state.transform,
                clip_rect: state.clip_rect.clone(),
            };

            if Some(&ins) != self.instructions.last() {
                self.instructions.push(ins);
            }
        }
        self
    }
}

pub(crate) struct BatchedGraphicsInstruction<'a> {
    pub path_instructions: &'a [GfxPathInstruction],
    pub transform: &'a Mat3,
    pub clip_rect: &'a Rect<f32>,
    pub fill: Option<&'a FillStyle>,
    pub stroke: Option<&'a StrokeStyle>,
}

pub(crate) struct BatchedGraphicsContextIter<'a> {
    context: &'a GraphicsContext,
    instructions: std::slice::Iter<'a, GraphicsInstruction>,
    // Store the next instruction to check for batching
    peeked: Option<&'a GraphicsInstruction>,
}

impl<'a> BatchedGraphicsContextIter<'a> {
    pub fn new(context: &'a GraphicsContext) -> Self {
        let mut instructions = context.instructions.iter();
        let peeked = instructions.next();
        Self {
            context,
            instructions,
            peeked,
        }
    }
}

impl<'a> Iterator for BatchedGraphicsContextIter<'a> {
    type Item = BatchedGraphicsInstruction<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the current instruction
        let current = self.peeked.take()?;
        let current_path = current.kind.path();
        let current_transform = &current.transform;
        let current_clip_rect = &current.clip_rect;

        // Look ahead to see if next instruction uses same geometry
        let next = self.instructions.next();
        let mut fill = None;
        let mut stroke = None;

        // Process current instruction
        match &current.kind {
            GraphicsInstructionKind::Fill { fill_style, .. } => {
                fill = Some(fill_style);
            }
            GraphicsInstructionKind::Stroke { stroke_style, .. } => {
                stroke = Some(stroke_style);
            }
        }

        // If next instruction uses same path, transform, and clip, combine it
        if let Some(next_inst) = next {
            let next_path = next_inst.kind.path();

            let same_geometry = next_path == current_path
                && next_inst.transform == *current_transform
                && next_inst.clip_rect == *current_clip_rect;

            if same_geometry {
                // Add the operation from next instruction
                match &next_inst.kind {
                    GraphicsInstructionKind::Fill { fill_style, .. } => {
                        if fill.is_none() {
                            fill = Some(fill_style);
                            // Continue with iterator - don't save next for peeking
                            self.peeked = self.instructions.next();
                        } else {
                            // Same operation type, don't batch
                            self.peeked = Some(next_inst);
                        }
                    }
                    GraphicsInstructionKind::Stroke { stroke_style, .. } => {
                        if stroke.is_none() {
                            stroke = Some(stroke_style);
                            // Continue with iterator - don't save next for peeking
                            self.peeked = self.instructions.next();
                        } else {
                            // Same operation type, don't batch
                            self.peeked = Some(next_inst);
                        }
                    }
                }
            } else {
                // Can't batch - different geometry
                self.peeked = Some(next_inst);
            }
        } else {
            // No more instructions
            self.peeked = None;
        }

        // Get path instructions for the current entry
        let path_instructions = self.context.path.get_entry(current_path);

        Some(BatchedGraphicsInstruction {
            path_instructions,
            transform: current_transform,
            clip_rect: current_clip_rect,
            fill,
            stroke,
        })
    }
}

#[cfg(test)]
mod tests {
    use ara_math::vec2;

    use super::*;

    #[test]
    fn test_graphic_context_push_instruction() {
        let mut cx = GraphicsContext::new();
        let color = Color::WHITE;

        cx.rect(Rect::xywh(0.0, 0.0, 100.0, 100.0));
        cx.fill(color);
        cx.stroke(color);

        cx.circle(vec2(100.0, 100.0), 50.0);
        cx.fill(color);
        cx.stroke(color);

        assert_eq!(cx.path.instructions.len(), 2);
        assert_eq!(cx.instructions.len(), 4);
        assert!(matches!(
            cx.instructions[0].kind,
            GraphicsInstructionKind::Fill {
                path: GfxPathEntry { start: 0, end: 1 },
                ..
            }
        ));
        assert!(matches!(
            cx.instructions[1].kind,
            GraphicsInstructionKind::Stroke {
                path: GfxPathEntry { start: 0, end: 1 },
                ..
            }
        ));
        assert!(matches!(
            cx.instructions[2].kind,
            GraphicsInstructionKind::Fill {
                path: GfxPathEntry { start: 1, end: 2 },
                ..
            }
        ));
        assert!(matches!(
            cx.instructions[3].kind,
            GraphicsInstructionKind::Stroke {
                path: GfxPathEntry { start: 1, end: 2 },
                ..
            }
        ));
    }
}
