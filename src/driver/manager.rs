use super::DriverDescriptor;
use crate::sync::NullLock;

pub struct DriverManagerInner<const N: usize> {
    drivers: [Option<DriverDescriptor>; N],
    idx: usize,
}

impl<const N: usize> DriverManagerInner<N> {
    const fn new() -> Self {
        Self {
            idx: 0,
            drivers: [const { None }; N],
        }
    }

    fn register_driver(&mut self, driver_descriptor: DriverDescriptor) {
        self.drivers[self.idx] = Some(driver_descriptor);
        self.idx += 1;
    }

    unsafe fn init(&mut self) {
        self.drivers
            .iter()
            .filter_map(|d| d.as_ref())
            .for_each(|d| {
                if let Err(s) = d.driver.init() {
                    panic!("Driver {} failed to initialize:\n{}", d.name, s)
                }

                if let Some(f) = d.post_init {
                    if let Err(s) = f() {
                        panic!("Driver {} failed to run post_init callback:\n{}", d.name, s)
                    }
                }
            });
    }
}

pub struct DriverManager<const N: usize> {
    inner: NullLock<DriverManagerInner<N>>,
}
impl<const N: usize> DriverManager<N> {
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(DriverManagerInner::new()),
        }
    }

    pub fn register_driver(&self, driver_descriptor: DriverDescriptor) {
        self.inner.lock(|i| i.register_driver(driver_descriptor));
    }
    pub unsafe fn init(&self) {
        self.inner.lock(|i| i.init());
    }
}
