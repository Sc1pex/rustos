use super::Driver;
use crate::{memory::mmio, sync::NullLock};

const BASE: u32 = 0xFE200000;
const GPFSEL0: u32 = BASE + 0;
const GPSET0: u32 = BASE + 0x1C;
const GPCLR0: u32 = BASE + 0x28;
const GPPUPDN0: u32 = BASE + 0xE4;

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

struct GPIODriverInner {}
impl GPIODriverInner {
    fn function(&self, pin: u32, val: Function) {
        self.write(pin, val as u32, GPFSEL0, 3);
    }

    fn resistor(&self, pin: u32, val: Resistor) {
        self.write(pin, val as u32, GPPUPDN0, 2);
    }

    fn set(&self, pin: u32) {
        self.write(pin, 1, GPSET0, 1);
    }

    fn clear(&self, pin: u32) {
        self.write(pin, 1, GPCLR0, 1);
    }

    fn write(&self, pin: u32, val: u32, base: u32, field_size: u32) {
        let field_mask = (1 << field_size) - 1;

        let num_fields = 32 / field_size;
        let reg = base + (pin / num_fields) * 4;
        let shift = (pin % num_fields) * field_size;

        let mut reg_val = mmio::read(reg);
        reg_val &= !(field_mask << shift);
        reg_val |= val << shift;
        mmio::write(reg, reg_val);
    }
}

pub struct GPIODriver {
    inner: NullLock<GPIODriverInner>,
}
impl GPIODriver {
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(GPIODriverInner {}),
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
