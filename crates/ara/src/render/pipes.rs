use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
};

use super::{AnyItem, Item, ItemManager, RenderContext};

#[derive(Clone)]
struct AnyPipe {
    item: AnyItem,
    init: fn(AnyItem, &mut RenderContext),
}

#[derive(Default)]
pub struct PipeCollection {
    pipe_item_map: BTreeMap<TypeId, AnyPipe>,
}

pub trait RenderPipe: Any {
    fn init(&mut self, cx: &mut RenderContext)
    where
        Self: Sized;
}

impl PipeCollection {
    pub fn init(cx: &mut RenderContext) {
        let pipes: Vec<_> = cx
            .pipes_collection
            .pipe_item_map
            .values()
            .cloned()
            .collect();

        for pipe in pipes {
            (pipe.init)(pipe.item, cx);
        }
    }

    pub fn get_handle<S: RenderPipe + 'static>(&self) -> Option<Item<S>> {
        let type_id = TypeId::of::<S>();
        self.pipe_item_map
            .get(&type_id)
            .and_then(|sys| sys.item.upgrade::<S>())
    }

    pub fn add<P: RenderPipe + 'static>(&mut self, handle: Item<P>) {
        let type_id = TypeId::of::<P>();

        if self.pipe_item_map.contains_key(&type_id) {
            log::warn!(
                "Pipe of type {:?} already exists",
                std::any::type_name::<P>()
            );
            return;
        }

        let any_item = handle.as_any_item();

        let any_pipe = AnyPipe {
            item: any_item,
            init: |item, cx| {
                let handle = item.upgrade::<P>().expect("Pipe type mismatch");
                cx.update_item::<P, ()>(&handle, |s, cx| {
                    s.init(cx);
                })
                .unwrap_or_else(|e| {
                    log::error!("Failed to init pipe: {:?}", e);
                })
            },
        };

        self.pipe_item_map.insert(type_id, any_pipe);
    }
}
