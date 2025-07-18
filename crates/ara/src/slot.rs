use std::{cell::RefCell, collections::BTreeMap, fmt::Debug, rc::Rc};

pub type SlotHandle = usize;

pub struct Slot<Callback: 'static> {
    inner: Rc<RefCell<SlotInner<Callback>>>,
    label: &'static str,
}

impl<Callback: 'static> Clone for Slot<Callback> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            label: self.label,
        }
    }
}

impl<Callback: 'static> Default for Slot<Callback> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Callback: 'static> Debug for Slot<Callback> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Slot").field("label", &self.label).finish()
    }
}

struct SlotInner<Callback: 'static> {
    subscriptions: BTreeMap<SlotHandle, Subscriber<Callback>>,
    next_id: SlotHandle,
}

impl<Callback: 'static> Default for SlotInner<Callback> {
    fn default() -> Self {
        Self {
            subscriptions: BTreeMap::new(),
            next_id: 1,
        }
    }
}

impl<Callback: 'static> Slot<Callback> {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
            label: "Slot",
        }
    }

    pub fn new_labelled(label: &'static str) -> Self {
        Self {
            inner: Default::default(),
            label,
        }
    }

    pub fn add(&self, cb: Callback) -> Subscription {
        let mut inner = self.inner.borrow_mut();
        let id = inner.next_id;

        let this = Rc::downgrade(&self.inner);

        inner.subscriptions.insert(id, Subscriber { callback: cb });
        inner.next_id += 1;

        let dispose = Subscription::new(move || {
            if let Some(inner) = this.upgrade() {
                let mut inner = inner.borrow_mut();
                inner.subscriptions.remove(&id);
            }
        });

        dispose
    }

    pub fn clear(&self) {
        let mut inner = self.inner.borrow_mut();
        inner.subscriptions.clear();
    }

    pub fn count(&self) -> usize {
        let inner = self.inner.borrow();
        inner.subscriptions.len()
    }

    pub fn emit_while(&self, mut callback: impl FnMut(&mut Callback) -> bool) {
        let mut inner = self.inner.borrow_mut();

        for (_, subscription) in inner.subscriptions.iter_mut() {
            if !callback(&mut subscription.callback) {
                break;
            }
        }
    }

    pub fn emit_ok<R, E>(
        &self,
        mut callback: impl FnMut(&mut Callback) -> Result<R, E>,
    ) -> Result<(), E> {
        let mut inner = self.inner.borrow_mut();

        for (_, subscription) in inner.subscriptions.iter_mut() {
            callback(&mut subscription.callback)?;
        }

        Ok(())
    }

    pub fn emit<F>(&self, mut callback: F)
    where
        F: FnMut(&mut Callback),
    {
        let mut inner = self.inner.borrow_mut();

        for (_, subscription) in inner.subscriptions.iter_mut() {
            callback(&mut subscription.callback);
        }
    }
}

struct Subscriber<Callback: 'static> {
    callback: Callback,
}

pub struct Subscription {
    dispose: Option<Box<dyn FnOnce() + 'static>>,
}

impl Subscription {
    pub fn new(dispose: impl FnOnce() + 'static) -> Self {
        Self {
            dispose: Some(Box::new(dispose)),
        }
    }

    /// this will detach the subscription from the slot and listen until the entity is dropped
    pub fn detach(mut self) {
        self.dispose.take();
    }

    pub fn join(mut self, mut other: Self) -> Self {
        let a_dispose = self.dispose.take();
        let b_dispose = other.dispose.take();
        Self::new(move || {
            if let Some(dispose) = a_dispose {
                dispose();
            }
            if let Some(dispose) = b_dispose {
                dispose();
            }
        })
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        if let Some(dispose) = self.dispose.take() {
            dispose();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_slot_basic() {
        let mut a = 1;
        let slot = Slot::new_labelled("test_slot");
        {
            let _dispose = slot.add(|x: &mut i32| {
                *x += 1;
            });

            assert_eq!(slot.count(), 1);

            slot.emit(|f| {
                f(&mut a);
            });

            assert_eq!(a, 2);
        }

        assert_eq!(slot.count(), 0);

        slot.emit(|f| {
            f(&mut a);
        });

        assert_eq!(a, 2);
    }

    #[test]
    fn test_slot_detach() {
        let mut a = 34;
        let slot = Slot::new();
        slot.add(|x: &mut i32, b: i32| {
            *x += b;
        })
        .detach();
        assert_eq!(slot.count(), 1);
        slot.emit(|f| {
            f(&mut a, 35);
        });
        assert_eq!(a, 69);
    }

    #[test]
    fn test_slot_subsription_join() {
        let mut a = 1;
        let s1 = Slot::new();
        let s2 = Slot::new();
        {
            let sub1 = s1.add(|x: &mut i32, b: i32| {
                *x += b;
            });
            let sub2 = s2.add(|x: &mut i32| {
                *x += 1;
            });

            assert_eq!(s1.count(), 1);
            assert_eq!(s2.count(), 1);

            let _join = Subscription::join(sub1, sub2);

            s1.emit(|f| {
                f(&mut a, 2);
            });
            s2.emit(|f| f(&mut a));
            assert_eq!(a, 4);
        }

        s1.emit(|f| {
            f(&mut a, 2);
        });
        s2.emit(|f| f(&mut a));

        assert_eq!(a, 4);
    }
}
