use std::ops::Range;

use ara_math::{Corners, Point, Rect};

use crate::{PathBuilder, PathEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum GfxPathInstruction {
    Rect {
        bounds: Rect<f32>,
    },
    RoundRect {
        bounds: Rect<f32>,
        corners: Corners<f32>,
    },
    Circle {
        center: Point,
        radius: f32,
    },
    Path {
        points: Range<usize>,
        verbs: Range<usize>,
    },
}

#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq)]
pub struct GfxPathEntry {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Default, Clone)]
pub struct GraphicsPath {
    pub(crate) builder: PathBuilder,
    pub(crate) paths: Vec<GfxPathEntry>,
    pub(crate) instructions: Vec<GfxPathInstruction>,
    instruction_start: usize,
}

impl GraphicsPath {
    pub fn clear(&mut self) {
        self.paths.clear();
        self.instructions.clear();
        self.instruction_start = 0;
    }

    pub(crate) fn get_entry(&self, entry: GfxPathEntry) -> &[GfxPathInstruction] {
        &self.instructions[entry.start..entry.end]
    }

    pub fn rect(&mut self, rect: Rect<f32>) {
        self.instructions
            .push(GfxPathInstruction::Rect { bounds: rect });
    }

    pub fn round_rect(&mut self, rect: Rect<f32>, corders: Corners<f32>) {
        self.instructions.push(GfxPathInstruction::RoundRect {
            bounds: rect,
            corners: corders,
        });
    }

    pub fn circle(&mut self, center: Point, radius: f32) {
        self.instructions
            .push(GfxPathInstruction::Circle { center, radius });
    }

    pub fn path2d<T>(&mut self, path: T)
    where
        T: IntoIterator<Item = PathEvent>,
    {
        let points_start = self.builder.points.len();
        let verbs_start = self.builder.verbs.len();
        self.builder.extend(path);
        let points_end = self.builder.points.len();
        let verbs_end = self.builder.verbs.len();

        self.instructions.push(GfxPathInstruction::Path {
            points: points_start..points_end,
            verbs: verbs_start..verbs_end,
        });
    }

    pub fn push(&mut self) -> Option<GfxPathEntry> {
        let start = self.instruction_start;
        let end = self.instructions.len();

        if start == end {
            return self.paths.last().cloned();
        }

        let p = GfxPathEntry { start, end };

        self.instruction_start = end;
        self.paths.push(p);
        Some(p)
    }
}
