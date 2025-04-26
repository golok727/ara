use std::ops::Range;

use crate::paint::GraphicsInstruction;

use super::CanvasState;

#[derive(Debug, Clone, PartialEq)]
struct StageItem {
    range: Range<usize>,
    state: CanvasState,
}

#[derive(Debug, Default, Clone)]
pub struct RenderList {
    pub(super) instructions: Vec<GraphicsInstruction>,
    stage: Vec<StageItem>,
}

impl RenderList {
    #[inline]
    pub fn add(&mut self, instruction: GraphicsInstruction) {
        self.instructions.push(instruction)
    }

    pub fn stage_changes(&mut self, state: CanvasState) {
        let start = self.stage
            .last()
            .map(|ins| ins.range.end)
            .unwrap_or(0);
        let end = self.instructions.len();

        if start < end {
            if let Some(last_stage) = self.stage.last_mut() {
                if last_stage.state == state {
                    last_stage.range = last_stage.range.start..end;
                    return;
                }
            }

            self.stage.push(StageItem {
                range: start..end,
                state,
            });
        }
    }

    #[inline]
    pub fn clear_staged(&mut self) {
        self.stage.clear()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.stage.clear();
        self.instructions.clear();
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }
}

pub struct RenderListIterItem<'a> {
    pub instructions: &'a [GraphicsInstruction],
    pub state: &'a CanvasState,
}

impl<'list> IntoIterator for &'list RenderList {
    type Item = RenderListIterItem<'list>;

    type IntoIter = RenderListIter<'list>;

    fn into_iter(self) -> Self::IntoIter {
        RenderListIter::new(self)
    }
}

pub struct RenderListIter<'list> {
    list: &'list RenderList,
    stages_iter: std::slice::Iter<'list, StageItem>,
}

impl<'list> RenderListIter<'list> {
    fn new(list: &'list RenderList) -> Self {
        let stages_iter = list.stage.iter();
        Self { list, stages_iter }
    }
}

impl<'list> Iterator for RenderListIter<'list> {
    type Item = RenderListIterItem<'list>;

    fn next(&mut self) -> Option<Self::Item> {
        self.stages_iter.next().map(|item| RenderListIterItem {
            instructions: &self.list.instructions[item.range.start..item.range.end],
            state: &item.state,
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.stages_iter.len();
        (len, Some(len))
    }
}

#[cfg(test)]
mod tests {
    use core::f32;

    use ara_math::Mat3;

    use crate::{ quad, Brush };

    use super::*;

    fn add_quad(list: &mut RenderList) {
        list.add(GraphicsInstruction::brush(quad().clone(), Brush::default()));
    }

    #[test]
    fn iterator() {
        let mut list = RenderList::default();
        add_quad(&mut list);
        add_quad(&mut list);
        add_quad(&mut list);

        list.stage_changes(CanvasState::default());
        add_quad(&mut list);
        add_quad(&mut list);
        let s2 = CanvasState {
            transform: Mat3::from_rotation(f32::consts::FRAC_PI_4),
            ..Default::default()
        };
        list.stage_changes(s2.clone());

        let mut iter = list.into_iter();
        let first = iter.next().expect("expected instructions");
        assert_eq!(first.instructions.len(), 3);
        assert_eq!(first.state, &CanvasState::default());

        let second = iter.next().expect("expected instructions");
        assert_eq!(second.instructions.len(), 2);
        assert_eq!(second.state, &s2);

        let next = iter.next();
        assert!(next.is_none());
    }

    #[test]
    fn render_list_basic() {
        let mut list = RenderList::default();
        add_quad(&mut list);
        add_quad(&mut list);
        add_quad(&mut list);

        assert_eq!(list.instructions.len(), 3);

        list.stage_changes(CanvasState::default());

        assert_eq!(list.stage.len(), 1);

        assert_eq!(
            &list.stage,
            &[
                StageItem {
                    range: 0..3,
                    state: CanvasState::default(),
                },
            ]
        )
    }

    #[test]
    fn stage_changes() {
        let mut list = RenderList::default();
        add_quad(&mut list);
        add_quad(&mut list);
        add_quad(&mut list);

        let s1 = CanvasState::default();
        list.stage_changes(s1.clone());

        add_quad(&mut list);
        add_quad(&mut list);

        let s2 = CanvasState {
            transform: Mat3::from_rotation(f32::consts::FRAC_PI_4),
            ..Default::default()
        };

        list.stage_changes(s2.clone());

        assert_eq!(
            &list.stage,
            &[
                StageItem {
                    range: 0..3,
                    state: s1,
                },
                StageItem {
                    range: 3..5,
                    state: s2,
                },
            ]
        )
    }

    #[test]
    fn no_stage_if_no_changes() {
        let mut list = RenderList::default();

        add_quad(&mut list);
        add_quad(&mut list);
        add_quad(&mut list);

        list.stage_changes(Default::default());
        list.stage_changes(Default::default());

        assert_eq!(
            &list.stage,
            &[
                StageItem {
                    range: 0..3,
                    state: CanvasState::default(),
                },
            ]
        )
    }

    #[test]
    fn extend_range_if_state_not_changed() {
        let mut list = RenderList::default();

        add_quad(&mut list);
        add_quad(&mut list);
        add_quad(&mut list);

        list.stage_changes(CanvasState::default());

        add_quad(&mut list);
        add_quad(&mut list);
        add_quad(&mut list);

        list.stage_changes(CanvasState::default());

        add_quad(&mut list);
        add_quad(&mut list);

        list.stage_changes(CanvasState::default());

        assert_eq!(
            &list.stage,
            &[
                StageItem {
                    range: 0..8,
                    state: CanvasState::default(),
                },
            ]
        )
    }

    #[test]
    fn all_cases() {
        let mut list = RenderList::default();
        add_quad(&mut list);
        add_quad(&mut list);
        add_quad(&mut list);

        // adds to state
        list.stage_changes(CanvasState::default());
        list.stage_changes(CanvasState::default());
        add_quad(&mut list); // extends from last
        list.stage_changes(CanvasState::default());

        add_quad(&mut list);
        add_quad(&mut list);
        list.stage_changes(CanvasState {
            transform: Mat3::from_translation(10.0, 10.0),
            ..Default::default()
        });
        list.stage_changes(CanvasState {
            transform: Mat3::from_translation(10.0, 10.0),
            ..Default::default()
        });
        list.stage_changes(CanvasState::default());

        add_quad(&mut list);
        add_quad(&mut list);

        list.stage_changes(CanvasState {
            transform: Mat3::from_translation(10.0, 10.0),
            ..Default::default()
        });

        add_quad(&mut list);
        list.stage_changes(CanvasState::default());

        // unstaged
        add_quad(&mut list);

        assert_eq!(
            &list.stage,
            &[
                StageItem {
                    range: 0..4,
                    state: CanvasState::default(),
                },
                StageItem {
                    range: 4..8,
                    state: CanvasState {
                        transform: Mat3::from_translation(10.0, 10.0),
                        ..Default::default()
                    },
                },
                StageItem {
                    range: 8..9,
                    state: CanvasState::default(),
                },
            ]
        )
    }

    #[test]
    fn is_empty() {
        let brush = Brush::default();
        let mut list = RenderList::default();

        assert!(list.is_empty());

        list.add(GraphicsInstruction::brush(quad(), brush.clone()));
        assert!(!list.is_empty());
    }
}
