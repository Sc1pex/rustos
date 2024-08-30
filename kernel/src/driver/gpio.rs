use super::Driver;
use crate::sync::NullLock;

#[allow(dead_code)]
pub enum Function {
    Output = 0b001,
    Alt0 = 0b100,
    Alt1 = 0b101,
    Alt2 = 0b110,
    Alt3 = 0b111,
    Alt4 = 0b011,
    Alt5 = 0b010,
}

#[allow(dead_code)]
pub enum Resistor {
    None = 0b00,
    Up = 0b01,
    Down = 0b10,
}

struct GPIODriverInner {
    gpfsel0: usize,
    gpset0: usize,
    gpclr0: usize,
    gppupdn0: usize,
}
impl GPIODriverInner {
    fn function(&self, pin: u32, val: Function) {
        self.write(pin, val as u32, self.gpfsel0, 3);
    }

    fn resistor(&self, pin: u32, val: Resistor) {
        self.write(pin, val as u32, self.gppupdn0, 2);
    }

    fn set(&self, pin: u32) {
        self.write(pin, 1, self.gpset0, 1);
    }

    fn clear(&self, pin: u32) {
        self.write(pin, 1, self.gpclr0, 1);
    }

    fn write(&self, pin: u32, val: u32, base: usize, field_size: u32) {
        let field_mask = (1 << field_size) - 1;

        let num_fields = 32 / field_size;
        let reg = base + ((pin / num_fields) * 4) as usize;
        let shift = (pin % num_fields) * field_size;

        let mut reg_val = mmio_read(reg);
        reg_val &= !(field_mask << shift);
        reg_val |= val << shift;
        mmio_write(reg, reg_val);
    }
}

pub struct GPIODriver {
    inner: NullLock<GPIODriverInner>,
}

#[allow(dead_code)]
impl GPIODriver {
    pub const fn new(base: usize) -> Self {
        Self {
            inner: NullLock::new(GPIODriverInner {
                gpfsel0: base + 0,
                gpset0: base + 0x1C,
                gpclr0: base + 0x28,
                gppupdn0: base + 0xE4,
            }),
        }
    }

    pub fn map_uart(&self) {
        self.inner.lock(|i| {
            i.function(14, Function::Alt0);
            i.function(15, Function::Alt0);
            i.resistor(14, Resistor::Up);
            i.resistor(15, Resistor::Up);
        });
    }
    pub fn function(&self, pin: u32, val: Function) {
        self.inner.lock(|i| i.function(pin, val));
    }
    pub fn resistor(&self, pin: u32, val: Resistor) {
        self.inner.lock(|i| i.resistor(pin, val));
    }
    pub fn set(&self, pin: u32) {
        self.inner.lock(|i| i.set(pin))
    }
    pub fn clear(&self, pin: u32) {
        self.inner.lock(|i| i.clear(pin))
    }
}
impl Driver for GPIODriver {
    unsafe fn init(&self) -> Result<(), &'static str> {
        Ok(())
    }
}

fn mmio_read(addr: usize) -> u32 {
    unsafe { core::ptr::read_volatile(addr as *const u32) }
}

fn mmio_write(addr: usize, val: u32) {
    unsafe {
        core::ptr::write_volatile(addr as *mut u32, val);
    }
}
