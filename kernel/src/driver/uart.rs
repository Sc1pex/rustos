use super::{Driver, MMIOWrapper};
use crate::{log::LogWrite, sync::NullLock};
use core::{arch::asm, fmt::Write};
use tock_registers::{
    interfaces::{Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

register_bitfields! {u32,
    /// Flag register
    FR [
        /// Transmit FIFO empty. The meaning of this bit depends on the state of the FEN bit in
        /// the Line Control Register,UART_LCRH.
        /// If the FIFO is disabled, this bit is set when the transmit holding register is empty.
        /// If the FIFO is enabled, the TXFE bit is set when the transmit FIFO is empty. This bit
        /// does not indicate if there is data in  the transmit shift register.
        TXFE OFFSET(7) NUMBITS(1),

        /// Receive FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// UART_LCRH Register.
        /// If the FIFO is disabled, this bit is set when the receive holding register is full.
        /// If the FIFO is enabled, the RXFF bit is set when the receive FIFO is full.
        RXFF OFFSET(6) NUMBITS(1),

        /// Transmit FIFO full. The meaning of this bit depends on the state of the FEN bit in the
        /// UART_LCRH Register.
        /// If the FIFO is disabled, this bit is set when the transmit holding register is full.
        ///If the FIFO is enabled, the TXFF bit is set when the transmit FIFO is full.
        TXFF OFFSET(5) NUMBITS(1),

        /// Receive FIFO empty. The meaning of this bit depends on the state of the FEN bit in the
        /// UART_LCRH Register.
        /// If the FIFO is disabled, this bit is set when the receive holding register is empty.
        /// If the FIFO is enabled, the RXFE bit is set when the receive FIFO is empty.
        RXFE OFFSET(4) NUMBITS(1),

        /// UART busy. If this bit is set to 1, the UART is busy transmitting data. This bit remains
        /// set until the complete byte, including all the stop bits, has been sent from the shift
        /// register.
        /// This bit is set as soon as the transmit FIFO becomes non-empty, regardless of whether
        /// the UART is enabled or not.
        BUSY OFFSET(3) NUMBITS(1),

        /// Clear to send. This bit is the complement of the UART clear to send, nUARTCTS, modem
        /// status input. That is, the bit is 1 when nUARTCTS is LOW.
        CTS OFFSET(0) NUMBITS(1),
    ],

    // Integer baud rate divisor
    IBRD [
        INT_BAUDDIV OFFSET(0) NUMBITS(16)
    ],

    // Fractional baud rate divisor
    FBRD [
        FRACT_BAUDDIV OFFSET(0) NUMBITS(6)
    ],

    // Line control register
    LCRH [
        /// Word length. These bits indicate the number of data bits transmitted or received
        WLEN OFFSET(5) NUMBITS(2) [
            Bits5 = 0b00,
            Bits6 = 0b01,
            Bits7 = 0b10,
            Bits8 = 0b11,
        ],

        /// Enable FIFOs:
        ///     0 = FIFOs are disabled (character mode) that is, the FIFOs become 1-byte-deep
        ///         holding registers
        ///     1 = transmit and receive FIFO buffers are enabled (FIFO mode).
        FEN OFFSET(4) NUMBITS(1) [
            Enable = 1,
            Disable = 0,
        ],
    ],

    // Control register
    CR [
        /// Receive enable. If this bit is set to 1, the receive section of the UART is enabled.
        RXE OFFSET(9) NUMBITS(1),

        /// Transmit enable. If this bit is set to 1, the transmit section of the UART is enabled
        TXE OFFSET(8) NUMBITS(1),

        /// UART enable:
        ///     0 = UART is disabled. If the UART is disabled in the middle of transmission or
        ///         reception, it completes the current character before stopping.
        ///     1 = the UART is enabled.
        EN OFFSET(0) NUMBITS(1)
    ],

    // Interrupt clear register
    ICR [
        ALL OFFSET(0) NUMBITS(11),
    ]
}

register_structs! {
    pub UartRegisters {
        (0x00 => dr: ReadWrite<u32>),
        (0x04 => _res1),
        (0x18 => fr: ReadOnly<u32, FR::Register>),
        (0x1c => _res2),
        (0x24 => ibrd: WriteOnly<u32, IBRD::Register>),
        (0x28 => fbrd: WriteOnly<u32, FBRD::Register>),
        (0x2c => lcrh: WriteOnly<u32, LCRH::Register>),
        (0x30 => cr: WriteOnly<u32, CR::Register>),
        (0x34 => _res3),
        (0x44 => icr: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

struct UARTDriverInner {
    regs: MMIOWrapper<UartRegisters>,
}
impl UARTDriverInner {
    fn init(&self) {
        let regs = &self.regs;

        regs.cr.write(CR::EN::CLEAR);
        regs.icr.write(ICR::ALL::CLEAR);

        // Set baudrate to 921600
        regs.ibrd.write(IBRD::INT_BAUDDIV.val(3));
        regs.fbrd.write(FBRD::FRACT_BAUDDIV.val(16));

        regs.lcrh.write(LCRH::FEN::Enable + LCRH::WLEN::Bits8);

        regs.cr.write(CR::EN::SET + CR::TXE::SET + CR::RXE::SET);
    }

    fn flush(&self) {
        while self.regs.fr.matches_all(FR::TXFF::SET) {
            unsafe {
                asm!("nop");
            }
        }
    }

    fn write(&mut self, c: u8) {
        while self.regs.fr.matches_all(FR::TXFF::SET) {
            unsafe {
                asm!("nop");
            }
        }

        self.regs.dr.set(c as u32);
    }

    fn read_blocking(&self) -> u8 {
        while self.regs.fr.matches_all(FR::RXFE::SET) {
            unsafe {
                asm!("nop");
            }
        }

        self.regs.dr.get() as u8
    }

    fn read(&self) -> Option<u8> {
        if self.regs.fr.matches_all(FR::RXFE::SET) {
            return None;
        }
        Some(self.read_blocking())
    }
}

impl Write for UARTDriverInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for c in s.bytes() {
            self.write(c);
        }
        Ok(())
    }
}

pub struct UARTDriver {
    inner: NullLock<UARTDriverInner>,
}

#[allow(dead_code)]
impl UARTDriver {
    pub const fn new(base: usize) -> Self {
        Self {
            inner: NullLock::new(UARTDriverInner {
                regs: MMIOWrapper::new(base),
            }),
        }
    }

    pub fn write_byte(&self, b: u8) {
        self.inner.lock(|i| i.write(b))
    }

    pub fn write_char(&self, c: char) {
        self.inner.lock(|i| {
            let mut b = [0; 4];
            let s = c.encode_utf8(&mut b);
            for c in s.bytes() {
                i.write(c);
            }
        });
    }
    pub fn write_str(&self, s: &str) {
        self.inner.lock(|i| i.write_str(s)).unwrap();
    }

    /// Block until the transmit FIFO is empty
    pub fn flush(&self) {
        self.inner.lock(|i| i.flush());
    }

    /// Block until a u8 is sent
    pub fn read_blocking(&self) -> u8 {
        self.inner.lock(|i| i.read_blocking())
    }
    /// Try to read a u8 without blocking
    pub fn read(&self) -> Option<u8> {
        self.inner.lock(|i| i.read())
    }

    /// Block until a char is sent
    pub fn read_char_blocking(&self) -> char {
        self.inner.lock(|i| {
            let mut b = [0; 4];
            for idx in 0..4 {
                b[idx] = i.read_blocking();

                if let Ok(c) = core::str::from_utf8(&b[0..idx]) {
                    // TODO: Check that str has lenght of 1?
                    return Self::map_char(c.chars().next().unwrap());
                }
            }

            // TODO: Return an error
            char::REPLACEMENT_CHARACTER
        })
    }
    /// Try to read a char
    /// If a multi byte char is read will block until all bytes are read
    pub fn read_char(&self) -> Option<char> {
        self.inner.lock(|i| {
            let mut b = [0; 4];
            b[0] = i.read()?;
            if let Ok(c) = core::str::from_utf8(&b[0..=1]) {
                // TODO: Check that str has lenght of 1?
                return Some(Self::map_char(c.chars().next().unwrap()));
            }

            for idx in 1..4 {
                b[idx] = i.read_blocking();

                if let Ok(c) = core::str::from_utf8(&b[0..=idx]) {
                    // TODO: Check that str has lenght of 1?
                    return Some(Self::map_char(c.chars().next().unwrap()));
                }
            }

            // TODO: Return an error
            None
        })
    }

    pub fn write_fmt(&self, args: core::fmt::Arguments) -> core::fmt::Result {
        self.inner.lock(|i| i.write_fmt(args))
    }

    fn map_char(c: char) -> char {
        match c {
            '\r' => '\n',
            _ => c,
        }
    }
}
impl Driver for UARTDriver {
    unsafe fn init(&self) -> Result<(), &'static str> {
        self.inner.lock(|i| i.init());
        Ok(())
    }
}

impl LogWrite for UARTDriver {
    fn write_str(&self, s: &str) {
        self.inner.lock(|i| i.write_str(s).unwrap())
    }
}
