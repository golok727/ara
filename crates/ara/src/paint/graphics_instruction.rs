use crate::{paint::Primitive, Brush, TextureId};
use std::{iter::Peekable, slice};

use super::Color;

// FIXME: seperate stuff with enum
#[derive(Debug, Clone)]
pub struct GraphicsInstruction {
    pub primitive: Primitive,
    pub brush: Brush,
    pub texture_id: TextureId,
}

impl GraphicsInstruction {
    pub fn nothing_to_draw(&self) -> bool {
        self.brush.nothing_to_draw()
    }

    pub fn textured(primitive: impl Into<Primitive>, texture_id: TextureId) -> Self {
        Self {
            primitive: primitive.into(),
            texture_id,
            brush: Brush::filled(Color::WHITE),
        }
    }

    pub fn brush(primitive: impl Into<Primitive>, brush: Brush) -> Self {
        Self {
            primitive: primitive.into(),
            texture_id: TextureId::WHITE_TEXTURE,
            brush,
        }
    }

    pub fn textured_brush(
        primitive: impl Into<Primitive>,
        texture_id: TextureId,
        brush: Brush,
    ) -> Self {
        Self {
            primitive: primitive.into(),
            texture_id,
            brush,
        }
    }
}

// batches instructions with the same texture
pub(crate) struct GraphicsInstructionBatcher<'a, TexMap>
where
    TexMap: Fn(&'a TextureId) -> Option<TextureId> + 'a,
{
    instruction_start: usize,
    instructions: &'a [GraphicsInstruction],
    instructions_iter: Peekable<slice::Iter<'a, GraphicsInstruction>>,
    get_renderer_texture: TexMap,
}

impl<'a, TexMap> GraphicsInstructionBatcher<'a, TexMap>
where
    TexMap: Fn(&'a TextureId) -> Option<TextureId> + 'a,
{
    /// # Arguments
    /// - `instructions` - A list of instructions to batch.
    /// - `get_renderer_texture` - A function that maps `instruction.texture` to the actual `texture_id`
    ///   bound to the renderer. Returns `None` if `instruction.texture` is already the actual texture ID
    ///   used in the renderer. Primarily used for atlas keys.
    pub fn new(instructions: &'a [GraphicsInstruction], get_renderer_texture: TexMap) -> Self {
        let instructions_iter = instructions.iter().peekable();

        Self {
            instruction_start: 0,
            instructions,
            instructions_iter,
            get_renderer_texture,
        }
    }
}

impl<'a, TexMap> Iterator for GraphicsInstructionBatcher<'a, TexMap>
where
    TexMap: Fn(&'a TextureId) -> Option<TextureId> + 'a,
{
    type Item = InstructionBatch<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.instruction_start >= self.instructions.len() {
            return None;
        }

        let first_instr = &self.instructions[self.instruction_start];
        let render_texture = (self.get_renderer_texture)(&first_instr.texture_id)
            .unwrap_or(first_instr.texture_id.clone());

        let mut end = self.instruction_start;

        while let Some(next_instr) = self.instructions_iter.peek() {
            let next_render_texture = (self.get_renderer_texture)(&next_instr.texture_id)
                .unwrap_or(next_instr.texture_id.clone());

            if next_render_texture != render_texture {
                break;
            }

            self.instructions_iter.next();
            end += 1;
        }

        let batch = InstructionBatch {
            instructions_iter: self.instructions[self.instruction_start..end].iter(),
            renderer_texture: render_texture,
        };

        self.instruction_start = end;
        Some(batch)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.instructions.len() - self.instruction_start;
        (remaining, Some(remaining))
    }
}

pub struct InstructionBatch<'a> {
    instructions_iter: std::slice::Iter<'a, GraphicsInstruction>,
    pub renderer_texture: TextureId,
}

impl<'a> Iterator for InstructionBatch<'a> {
    type Item = &'a GraphicsInstruction;

    fn next(&mut self) -> Option<Self::Item> {
        self.instructions_iter.next()
    }
}

#[cfg(test)]
mod tests {
    use crate::{quad, TextureId};

    use super::{GraphicsInstruction, GraphicsInstructionBatcher};

    fn get_instructions() -> [GraphicsInstruction; 12] {
        [
            GraphicsInstruction::textured(quad(), TextureId::User(1)),
            GraphicsInstruction::textured(quad(), TextureId::User(1)),
            GraphicsInstruction::textured(quad(), TextureId::User(1)),
            GraphicsInstruction::textured(quad(), TextureId::WHITE_TEXTURE),
            GraphicsInstruction::textured(quad(), TextureId::User(2)),
            GraphicsInstruction::textured(quad(), TextureId::WHITE_TEXTURE),
            GraphicsInstruction::textured(quad(), TextureId::WHITE_TEXTURE),
            GraphicsInstruction::textured(quad(), TextureId::WHITE_TEXTURE),
            GraphicsInstruction::textured(quad(), TextureId::User(2)),
            GraphicsInstruction::textured(quad(), TextureId::User(2)),
            GraphicsInstruction::textured(quad(), TextureId::User(2)),
            GraphicsInstruction::textured(quad(), TextureId::WHITE_TEXTURE),
        ]
    }

    #[test]
    fn test_batcher() {
        let instructions = get_instructions();
        let batcher = GraphicsInstructionBatcher::new(&instructions, |_| None);
        let mut iter = batcher.into_iter();

        let mut test_next_exists = |id: TextureId, len: usize| {
            let next = iter.next().expect("batch not found");
            assert_eq!(next.renderer_texture, id);
            let vec: Vec<_> = next.collect();
            assert_eq!(vec.len(), len);
        };

        test_next_exists(TextureId::User(1), 3);
        test_next_exists(TextureId::WHITE_TEXTURE, 1);
        test_next_exists(TextureId::User(2), 1);
        test_next_exists(TextureId::WHITE_TEXTURE, 3);
        test_next_exists(TextureId::User(2), 3);
        test_next_exists(TextureId::WHITE_TEXTURE, 1);

        assert!(iter.next().is_none());
    }

    #[test]
    fn test_render_texture() {
        let instructions = get_instructions();
        let batcher = GraphicsInstructionBatcher::new(&instructions, |tex| {
            if tex == &TextureId::User(1) {
                Some(TextureId::Internal(1))
            } else {
                None
            }
        });

        let mut iter = batcher.into_iter();

        let mut test_next_exists = |id: TextureId, len: usize| {
            let next = iter.next().expect("batch not found");
            assert_eq!(next.renderer_texture, id);
            let vec: Vec<_> = next.collect();
            assert_eq!(vec.len(), len);
        };

        test_next_exists(TextureId::Internal(1), 3);
        test_next_exists(TextureId::WHITE_TEXTURE, 1);
        test_next_exists(TextureId::User(2), 1);
        test_next_exists(TextureId::WHITE_TEXTURE, 3);
        test_next_exists(TextureId::User(2), 3);
        test_next_exists(TextureId::WHITE_TEXTURE, 1);

        assert!(iter.next().is_none());
    }
}
