use std::{
    any::Any,
    cell::RefCell,
    fmt,
    hash::{Hash, Hasher},
    rc::{Rc, Weak},
};

use derive_more::derive::{Deref, DerefMut};
use slotmap::{SecondaryMap, SlotMap};

use super::{ItemContext, ItemManager, RenderContext};

slotmap::new_key_type! {
    pub struct ItemId;
}

#[derive(Deref, DerefMut)]
pub struct ItemSlot<T: 'static>(Item<T>);

pub struct ItemMap {
    registry: SecondaryMap<ItemId, Box<dyn Any>>,
    ref_counts: Rc<RefCell<ItemRefCounts>>,
}

impl Default for ItemMap {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemMap {
    pub fn new() -> Self {
        ItemMap {
            registry: SecondaryMap::new(),
            ref_counts: Rc::new(RefCell::new(ItemRefCounts::new())),
        }
    }
}

impl fmt::Debug for ItemMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ItemMap")
            .field("registry_size", &self.registry.len())
            .finish()
    }
}

struct ItemRefCounts {
    counts: SlotMap<ItemId, usize>,
}

impl ItemRefCounts {
    pub fn new() -> Self {
        ItemRefCounts {
            counts: SlotMap::with_key(),
        }
    }
}

impl ItemRefCounts {
    pub fn increment_ref_count(&mut self, id: ItemId) {
        if let Some(count) = self.counts.get_mut(id) {
            *count += 1;
        }
    }

    pub fn decrement_ref_count(&mut self, id: ItemId) {
        if let Some(count) = self.counts.get_mut(id) {
            if *count > 0 {
                *count -= 1;
            }
        }
    }
}

pub struct Item<T: 'static> {
    id: ItemId,
    any_item: AnyItem,
    _item_type: std::marker::PhantomData<T>,
}

impl<T: 'static> PartialEq for Item<T> {
    fn eq(&self, other: &Self) -> bool {
        other.id == self.id
    }
}

impl<T: 'static> Eq for Item<T> {}

impl<T: 'static> Item<T> {
    fn new(id: ItemId, ref_counts: Weak<RefCell<ItemRefCounts>>) -> Self {
        Item {
            id,
            any_item: AnyItem {
                id,
                item_type: std::any::TypeId::of::<T>(),
                ref_counts,
            },
            _item_type: std::marker::PhantomData,
        }
    }

    pub fn as_any_item(&self) -> AnyItem {
        self.any_item.clone()
    }

    pub fn update<C: ItemManager, R>(
        &self,
        cx: &mut C,
        update: impl FnOnce(&mut T, &mut ItemContext<T>) -> R,
    ) -> anyhow::Result<R> {
        cx.update_item(self, update)
    }

    pub fn read<C: ItemManager, R>(
        &self,
        cx: &C,
        read: impl FnOnce(&T, &RenderContext) -> R,
    ) -> anyhow::Result<R> {
        cx.read_item(self, |item, cx| read(item, cx))
    }

    pub fn downgrade(&self) -> WeakItem<T> {
        WeakItem {
            any_item: self.any_item.clone(),
            marker: std::marker::PhantomData,
        }
    }
}

impl<T: 'static> Drop for Item<T> {
    fn drop(&mut self) {
        if let Some(ref_counts) = self.any_item.ref_counts.upgrade() {
            ref_counts
                .borrow_mut()
                .decrement_ref_count(self.any_item.id);
        }
    }
}

impl<T: 'static> Clone for Item<T> {
    fn clone(&self) -> Self {
        if let Some(ref_counts) = self.any_item.ref_counts.upgrade() {
            ref_counts
                .borrow_mut()
                .increment_ref_count(self.any_item.id);
        }
        Self::new(self.id, self.any_item.ref_counts.clone())
    }
}

impl<T: 'static> Hash for Item<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<T: 'static> fmt::Display for Item<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Item<{}>(id: {:?})", std::any::type_name::<T>(), self.id)
    }
}

impl<T: 'static> fmt::Debug for Item<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Item")
            .field("id", &self.id)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

#[derive(Clone)]
pub struct AnyItem {
    pub id: ItemId,
    pub item_type: std::any::TypeId,
    ref_counts: Weak<RefCell<ItemRefCounts>>,
}

impl AnyItem {
    pub fn upgrade<T: 'static>(&self) -> Option<Item<T>> {
        if self.item_type == std::any::TypeId::of::<T>() {
            Some(Item::new(self.id, self.ref_counts.clone()))
        } else {
            None
        }
    }
}

impl fmt::Display for AnyItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AnyItem(id: {:?}, type: {:?})", self.id, self.item_type)
    }
}

impl fmt::Debug for AnyItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AnyItem")
            .field("id", &self.id)
            .field("item_type", &self.item_type)
            .finish()
    }
}

pub struct WeakItem<T: 'static> {
    any_item: AnyItem,
    marker: std::marker::PhantomData<T>,
}

impl<T: 'static> WeakItem<T> {
    pub fn upgrade(&self) -> Option<Item<T>> {
        self.any_item.upgrade::<T>()
    }
}

impl<T: 'static> fmt::Display for WeakItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WeakItem<{}>(id: {:?})",
            std::any::type_name::<T>(),
            self.any_item.id
        )
    }
}

impl<T: 'static> fmt::Debug for WeakItem<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WeakItem")
            .field("id", &self.any_item.id)
            .field("type", &std::any::type_name::<T>())
            .finish()
    }
}

impl ItemMap {
    pub fn has<T: 'static>(&self) -> bool {
        todo!()
    }

    fn assert_valid_context(&self, entity: &AnyItem) {
        debug_assert!(
            Weak::ptr_eq(&entity.ref_counts, &Rc::downgrade(&self.ref_counts)),
            "used a item with the wrong context"
        );
    }

    pub fn lease<'a, T: 'static>(&mut self, resource_ref: &'a Item<T>) -> ItemLease<'a, T> {
        let resource = Some(self.registry.remove(resource_ref.id).unwrap_or_else(|| {
            panic!(
                "Can't perform lease operation while updating resource {}",
                std::any::type_name::<T>()
            )
        }));
        ItemLease {
            resource,
            resource_ref,
        }
    }

    pub fn end_lease<T: 'static>(&mut self, mut lease: ItemLease<T>) {
        self.registry
            .insert(lease.resource_ref.id, lease.resource.take().unwrap());
    }

    pub fn reserve<T: 'static>(&self) -> ItemSlot<T> {
        let mut refs = self.ref_counts.borrow_mut();

        let id = refs.counts.insert(0);
        refs.increment_ref_count(id);
        ItemSlot(Item::new(id, Rc::downgrade(&self.ref_counts)))
    }

    #[track_caller]
    pub fn insert<T: 'static>(&mut self, slot: ItemSlot<T>, resource: T) -> Item<T> {
        self.assert_valid_context(&slot.0.any_item);

        let id = slot.0.id;
        self.registry.insert(id, Box::new(resource));
        self.ref_counts.borrow_mut().increment_ref_count(id);

        slot.0
    }

    pub fn read<T: 'static>(&self, handle: &Item<T>) -> &T {
        self.assert_valid_context(&handle.any_item);
        self.registry
            .get(handle.id)
            .and_then(|v| v.downcast_ref::<T>())
            .unwrap_or_else(|| {
                panic!(
                    "Can't perform read operation while updating resource {}",
                    std::any::type_name::<T>()
                )
            })
    }

    pub fn remove<T: 'static>(&mut self, handle: &Item<T>) -> Option<T> {
        let res = self
            .registry
            .remove(handle.id)
            .and_then(|v| v.downcast::<T>().ok())
            .map(|v| *v);

        if res.is_some() {
            self.ref_counts.borrow_mut().decrement_ref_count(handle.id);
        }

        res
    }
}

impl<T: 'static> fmt::Display for ItemSlot<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "ItemSlot<{}>(id: {:?})",
            std::any::type_name::<T>(),
            self.0.id
        )
    }
}

impl<T: 'static> fmt::Debug for ItemSlot<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ItemSlot").field("item", &self.0).finish()
    }
}

pub struct ItemLease<'a, T: 'static> {
    resource: Option<Box<dyn Any>>,
    resource_ref: &'a Item<T>,
}

impl<'a, T: 'static> std::fmt::Display for ItemLease<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ItemLease({})", std::any::type_name::<T>())
    }
}

impl<'a, T: 'static> core::ops::Drop for ItemLease<'a, T> {
    fn drop(&mut self) {
        if self.resource.is_some() {
            panic!(
                "lease should be released with ItemLease::end(){}",
                std::any::type_name::<T>()
            );
        }
    }
}

impl<'a, T: 'static> core::ops::Deref for ItemLease<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.resource
            .as_ref()
            .and_then(|v| v.downcast_ref::<T>())
            .unwrap()
    }
}
impl<'a, T: 'static> core::ops::DerefMut for ItemLease<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.resource
            .as_mut()
            .and_then(|v| v.downcast_mut::<T>())
            .unwrap()
    }
}

#[cfg(test)]
mod test {}
