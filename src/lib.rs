// implementing the refcell type
pub mod RefCell {
    
use std::cell::UnsafeCell;
use std::cell::Cell;    
pub struct RefCell<T> {
    value: UnsafeCell<T>,
    tracker: Cell<isize>,
}

#[derive(Copy, Clone)]
enum Tracker {
    unshared,
    shared(usize),
    sharedmut,
}

impl <T> RefCell<T> {
    fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(T),
            tracker: Cell::new(Tracker::unshared),   // since we are mutating tracker through shared references
        }                                            // we can just make it into a cell
    }

    pub fn borrow(&self) -> Option<Ref<'_, T>> {
        match self.tracker.get() {
            Tracker::unshared => {
                self.tracker.set(Tracker::shared(1));
                Some(Ref{
                    refcell: self,
                })
            }
            Tracker::shared(n) => {
                self.tracker.set(Tracker::shared(n+1));
                Some(Ref{
                    refcell: self,
                })
            }
            Tracker::sharedmut => {
                None
            }
        }
    }

    pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
        if let Tracker::unshared = self.tracker.get() {
            Some(RefMut{
                refcell: self,
            })
        }
        else {
            None
        }
    }
}

// the problem upto here is once the tracker is set to unsharedmut there is no way for me to get a reference
// even if i drop the mutable reference... so certain implementations need to be made
// we introduce a new type as there is no way to directly change the state of the Tracker once things are dropped

pub struct Ref<'refcell, T> {
    refcell : &'refcell RefCell<T>,
}

impl <T> Drop for Ref<'_ , T> {
    fn drop(&mut self) {
        match self.refcell.tracker.get() {
            Tracker::sharedmut => unreachable!(),
            Tracker::unshared => unreachable!(),
            Tracker::shared(1) => {
                self.refcell.tracker.set(Tracker::unshared);
            }
            Tracker::shared(n) => {
                self.refcell.tracker.set(Tracker::shared(n-1));
            }
        }
    }
}

impl <T> std::ops::Deref for Ref<'_ T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe{
            &*self.refcell.tracker.get()
        }
    }
}

pub struct RefMut<'refcell, T> {
    refcell: &'refcell RefCell<T>,
}

impl <T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        match self.refcell.tracker.get() {
            Tracker::shared(_) => unreachable!(),
            Tracker::unshared => unreachable!(),
            Tracker::sharedmut => {
                self.refcell.tracker.set(Tracker::unshared);
            }
        }
    }
}

impl <T> std::ops::Deref for RefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe{
            *self.refcell.tracker.get()
        }
    }
}

//we also have to implement DerefMut trait for the RefMut type
// we cant implement DerefMut for the Ref type because that can lead to multiple mutable references to 
//the same type

impl <T> std::ops::DerefMut for RefMut<'_, T> {
    type Target = T;
    fn derefmut(&mut self) -> &mut Self::Target {
        unsafe{
            &mut *self.refcell.tracker.get()
        }
    }
}
}
