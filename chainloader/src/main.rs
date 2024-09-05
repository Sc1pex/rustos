#![feature(format_args_nl)]
#![no_std]
#![no_main]

use core::panic::PanicInfo;
use uart::Uart;

mod boot;
mod gpio;
mod mmio;
mod uart;

pub static UART: Uart = Uart::new();

unsafe fn kernel_init() -> ! {
    gpio::map_uart();
    UART.init();

    kernel_start()
}

#[derive(Clone, Copy, Debug)]
enum ChunkKind {
    RLE,
    Normal,
}
#[derive(Debug)]
struct Chunk<'a> {
    kind: ChunkKind,
    data: &'a mut [u8],
}

impl TryFrom<u8> for ChunkKind {
    type Error = &'static str;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'R' => Ok(Self::RLE),
            b'N' => Ok(Self::Normal),
            _ => Err("Unknown kind"),
        }
    }
}

impl Into<u8> for ChunkKind {
    fn into(self) -> u8 {
        match self {
            ChunkKind::RLE => b'R',
            ChunkKind::Normal => b'N',
        }
    }
}

impl<'a> Chunk<'a> {
    fn from_uart(buf: &'a mut [u8]) -> Result<Self, &'static str> {
        let kind: ChunkKind = UART.read_blocking().try_into()?;
        let mut len = [0; 4];
        for i in 0..4 {
            len[i] = UART.read_blocking();
        }
        let len = u32::from_le_bytes(len) as usize;

        Ok(Self {
            kind,
            data: &mut buf[0..len],
        })
    }

    fn read_data(&mut self) {
        for d in self.data.iter_mut() {
            *d = UART.read_blocking();
        }
    }

    fn verify_data(&self) -> bool {
        UART.write(self.kind.into());
        UART.write_slice(&(self.data.len() as u32).to_le_bytes());
        for d in self.data.iter() {
            UART.write(*d);
        }

        UART.read_blocking() == b'G'
    }

    /// Returns number of bytes written to data
    fn write(&self, data: &mut [u8]) -> usize {
        match self.kind {
            ChunkKind::Normal => {
                let data = &mut data[0..self.data.len()];
                data.copy_from_slice(self.data);
                self.data.len()
            }
            ChunkKind::RLE => {
                let mut offset = 0;
                for b in self.data.chunks_exact(2) {
                    for i in 0..b[0] as usize {
                        data[offset + i] = b[1];
                    }
                    offset += b[0] as usize;
                }
                offset
            }
        }
    }
}

fn kernel_start() -> ! {
    write!(UART, "Requesting binary\n");
    write!(UART, "\0\0\0");

    let mut kernel_size = [0; 4];
    for i in 0..4 {
        kernel_size[i] = UART.read_blocking();
    }
    let kernel_size = u32::from_le_bytes(kernel_size) as usize;

    let kernel_addr = 0x80000 as *mut u8;
    let kernel_data: &mut [u8] =
        unsafe { core::slice::from_raw_parts_mut(kernel_addr, kernel_size as usize) };

    const MAX_BLOCK_SIZE: usize = 1024;
    let mut read_buf = [0; MAX_BLOCK_SIZE];

    let mut start = 0;
    while start < kernel_size {
        let chunk = loop {
            let mut chunk = Chunk::from_uart(&mut read_buf).unwrap();
            chunk.read_data();

            if chunk.verify_data() {
                break chunk;
            }
        };

        let cnt = chunk.write(&mut kernel_data[start..]);
        start += cnt;
    }

    let kernel: fn() -> ! = unsafe { core::mem::transmute(kernel_addr) };
    kernel()
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = write!(UART, "KERNEL PANIC!: {}\n", info.message());
    if let Some(loc) = info.location() {
        let _ = write!(UART, "in file {}:{}", loc.file(), loc.line());
    }

    loop {}
}
