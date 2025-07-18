mod encoder;
mod geometry;
mod global_uniform;

use std::{
    any::{Any, TypeId},
    collections::BTreeMap,
};

pub use encoder::*;
pub use geometry::*;
pub use global_uniform::*;

use super::{AnyItem, Item, ItemManager, RenderContext};

#[derive(Clone)]
struct AnySystem {
    item: AnyItem,
    init: fn(AnyItem, &mut RenderContext),
}

#[derive(Default, Clone)]
pub struct SystemCollection {
    system_item_map: BTreeMap<TypeId, AnySystem>,
}

impl SystemCollection {
    pub fn get_handle<S: System + 'static>(&self) -> Option<Item<S>> {
        let type_id = TypeId::of::<S>();
        self.system_item_map
            .get(&type_id)
            .and_then(|sys| sys.item.upgrade::<S>())
    }

    pub fn add<S: System + 'static>(&mut self, handle: Item<S>) {
        let type_id = TypeId::of::<S>();

        if self.system_item_map.contains_key(&type_id) {
            log::warn!(
                "System of type {:?} already exists",
                std::any::type_name::<S>()
            );
            return;
        }

        let any_item = handle.as_any_item();

        let any_system = AnySystem {
            item: any_item,
            init: |item, cx| {
                let handle = item.upgrade::<S>().expect("System type mismatch");
                cx.update_item::<S, ()>(&handle, |s, cx| {
                    s.init(cx);
                })
                .unwrap_or_else(|e| {
                    log::error!("Failed to init system: {:?}", e);
                })
            },
        };
        self.system_item_map.insert(type_id, any_system);
    }

    pub fn init(cx: &mut RenderContext) {
        let systems: Vec<_> = cx
            .systems_collection
            .system_item_map
            .values()
            .cloned()
            .collect();

        for system in systems {
            (system.init)(system.item, cx);
        }
    }
}

pub trait System: Any {
    fn init(&mut self, cx: &mut RenderContext)
    where
        Self: Sized;
}

pub struct HelloSystem;

impl System for HelloSystem {
    fn init(&mut self, _cx: &mut RenderContext) {
        log::debug!("HelloSystem: Hello Welcome to Ara!");
    }
}
