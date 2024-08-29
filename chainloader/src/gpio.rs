use crate::mmio;

const BASE: u32 = 0xFE200000;
const GPFSEL0: u32 = BASE + 0;
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

fn function(pin: u32, val: Function) {
    write(pin, val as u32, GPFSEL0, 3);
}

fn resistor(pin: u32, val: Resistor) {
    write(pin, val as u32, GPPUPDN0, 2);
}

fn write(pin: u32, val: u32, base: u32, field_size: u32) {
    let field_mask = (1 << field_size) - 1;

    let num_fields = 32 / field_size;
    let reg = base + (pin / num_fields) * 4;
    let shift = (pin % num_fields) * field_size;

    let mut reg_val = mmio::read(reg);
    reg_val &= !(field_mask << shift);
    reg_val |= val << shift;
    mmio::write(reg, reg_val);
}

pub fn map_uart() {
    function(14, Function::Alt0);
    function(15, Function::Alt0);
    resistor(14, Resistor::Up);
    resistor(15, Resistor::Up);
}
