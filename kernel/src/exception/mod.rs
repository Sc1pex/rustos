use aarch64_cpu::{
    asm::barrier,
    registers::{Readable, Writeable, ESR_EL1, FAR_EL1, SPSR_EL1, VBAR_EL1},
};
use core::{
    arch::{asm, global_asm},
    fmt::Display,
};
use tock_registers::registers::InMemoryRegister;

global_asm!(include_str!("exception.S"));

#[derive(Debug)]
pub enum PrivilegeLevel {
    Application,
    Kernel,
    Hypervisor,
    Unknown,
}

pub fn current_el() -> PrivilegeLevel {
    let el = unsafe {
        let x: u64;
        asm!("mrs {}, CurrentEL", out(reg) x);
        x
    };
    match el {
        0b0000 => PrivilegeLevel::Application,
        0b0100 => PrivilegeLevel::Kernel,
        0b1000 => PrivilegeLevel::Hypervisor,
        _ => PrivilegeLevel::Unknown,
    }
}

pub unsafe fn init_handlers() {
    use core::cell::UnsafeCell;
    extern "Rust" {
        static __exception_vector_start: UnsafeCell<()>;
    }

    VBAR_EL1.set(__exception_vector_start.get() as u64);
    barrier::isb(barrier::SY)
}

#[repr(C)]
struct ExceptionContext {
    regs: [u64; 30],
    lr: u64,

    elr_el1: u64,

    spsr_el1: InMemoryRegister<u64, SPSR_EL1::Register>,
    esr_el1: InMemoryRegister<u64, ESR_EL1::Register>,
}

fn default_exception_handler(ctx: &ExceptionContext) {
    panic!("CPU Exception:\n{}", ctx);
}

impl Display for ExceptionContext {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "\nESR_EL1: {:#018X}", self.esr_el1.get())?;
        writeln!(
            f,
            "    Exception class: {:#02X}",
            self.esr_el1.read(ESR_EL1::EC)
        )?;
        writeln!(
            f,
            "    Instr specific syndrome: {:X}",
            self.esr_el1.read(ESR_EL1::EC)
        )?;

        if self.has_fault_addr() {
            writeln!(f, "FAR_EL1: {:#018X}", FAR_EL1.get())?;
        }

        let bool_str = |x: bool, s1: &'static str, s2: &'static str| {
            if x {
                s1
            } else {
                s2
            }
        };
        writeln!(f, "SPSR_EL1: {:#018X}", self.spsr_el1.get())?;
        writeln!(f, "    Flags:")?;
        writeln!(
            f,
            "      Negative: {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::N), "Set", "Not set")
        )?;
        writeln!(
            f,
            "      Zero:     {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::Z), "Set", "Not set")
        )?;
        writeln!(
            f,
            "      Carry:    {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::C), "Set", "Not set")
        )?;
        writeln!(
            f,
            "      Overflow: {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::V), "Set", "Not set")
        )?;

        writeln!(f, "    Exception masking:")?;
        writeln!(
            f,
            "      Debug:  {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::D), "Masked", "Not masked")
        )?;
        writeln!(
            f,
            "      SError: {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::A), "Masked", "Not masked")
        )?;
        writeln!(
            f,
            "      IRQ:    {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::I), "Masked", "Not masked")
        )?;
        writeln!(
            f,
            "      FIQ:    {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::F), "Masked", "Not masked")
        )?;
        writeln!(
            f,
            "    Illegal execution state: {}",
            bool_str(self.spsr_el1.is_set(SPSR_EL1::IL), "Set", "Not set")
        )?;

        writeln!(f, "ELR_EL1: {:#018X}", self.elr_el1)?;

        writeln!(f, "\nGeneral purpose registers:")?;
        for i in (0..15).map(|i| i * 2) {
            writeln!(
                f,
                "    x{:<2}: {:#018X}      x{:<2}: {:#018X}",
                i,
                self.regs[i],
                i + 1,
                self.regs[i + 1]
            )?
        }

        Ok(())
    }
}

impl ExceptionContext {
    fn exception_class(&self) -> Option<ESR_EL1::EC::Value> {
        self.esr_el1.read_as_enum(ESR_EL1::EC)
    }

    fn has_fault_addr(&self) -> bool {
        use ESR_EL1::EC::Value::*;

        match self.exception_class() {
            None => false,
            Some(ec) => matches!(
                ec,
                InstrAbortLowerEL
                    | InstrAbortCurrentEL
                    | PCAlignmentFault
                    | DataAbortLowerEL
                    | DataAbortCurrentEL
                    | WatchpointLowerEL
                    | WatchpointCurrentEL
            ),
        }
    }
}

// Exceptions from current EL while using SP_EL0
#[no_mangle]
extern "C" fn current_el0_sync(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn current_el0_irq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn current_el0_fiq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn current_el0_serror(e: &mut ExceptionContext) {
    default_exception_handler(e)
}

// Exceptions from cuurrent EL while using SP_ELx, x != 0
#[no_mangle]
extern "C" fn current_elx_sync(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn current_elx_irq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn current_elx_fiq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn current_elx_serror(e: &mut ExceptionContext) {
    default_exception_handler(e)
}

// Exceptions from a lowe EL, AArch64
#[no_mangle]
extern "C" fn lower_el_aarch64_sync(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn lower_el_aarch64_irq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn lower_el_aarch64_fiq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn lower_el_aarch64_serror(e: &mut ExceptionContext) {
    default_exception_handler(e)
}

// Exceptions from a lowe EL, AArch32
// These are probably impossible
#[no_mangle]
extern "C" fn lower_el_aarch32_sync(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn lower_el_aarch32_irq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn lower_el_aarch32_fiq(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
#[no_mangle]
extern "C" fn lower_el_aarch32_serror(e: &mut ExceptionContext) {
    default_exception_handler(e)
}
