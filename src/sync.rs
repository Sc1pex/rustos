use core::cell::UnsafeCell;

pub struct NullLock<T>
where
    T: ?Sized,
{
    data: UnsafeCell<T>,
}

unsafe impl<T> Send for NullLock<T> where T: ?Sized + Send {}
unsafe impl<T> Sync for NullLock<T> where T: ?Sized + Send {}

impl<T> NullLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let data = unsafe { &mut *self.data.get() };
        f(data)
    }
}
