pub mod gpio;
pub mod manager;
pub mod uart;

pub trait Driver {
    unsafe fn init(&self) -> Result<(), &'static str>;
}

pub struct DriverDescriptor {
    pub name: &'static str,
    pub driver: &'static (dyn Driver + Sync),
    pub post_init: Option<unsafe fn() -> Result<(), &'static str>>,
}
