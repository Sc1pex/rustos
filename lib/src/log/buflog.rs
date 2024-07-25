use crate::sync::NullLock;

use super::{LogLevel, LogWrite};
use core::fmt::Write;

const BUF_SIZE: usize = 1024;

struct BufLoggerInner {
    buf: [u8; BUF_SIZE],
    idx: usize,

    writer: Option<&'static (dyn LogWrite + Sync)>,
}

impl BufLoggerInner {
    const fn new() -> Self {
        Self {
            buf: [0; BUF_SIZE],
            idx: 0,

            writer: None,
        }
    }

    fn flush(&mut self) {
        match self.writer {
            // TODO: panic?
            None => (),
            Some(w) => {
                let s = core::str::from_utf8(&self.buf[0..self.idx]).unwrap();
                w.write_str(s);
            }
        }
        self.idx = 0;
    }

    fn write_buf(&mut self, b: &[u8]) {
        let to_copy = b.len().min(BUF_SIZE - self.idx);

        let b_dst = &mut self.buf[self.idx..self.idx + to_copy];
        b_dst.copy_from_slice(&b[0..to_copy]);
        self.idx += to_copy;
        self.check_and_flush();

        if to_copy > b.len() {
            self.write_buf(&b[to_copy..]);
        }
    }

    fn check_and_flush(&mut self) {
        if self.idx == BUF_SIZE {
            self.flush();
            return;
        }
        if let Some((last_nl, _)) = self.buf[0..self.idx]
            .iter()
            .enumerate()
            .rfind(|(_, c)| **c == b'\n')
        {
            if self.writer.is_none() {
                return;
            }

            let old_idx = self.idx;
            self.idx = last_nl + 1;
            self.flush();
            self.buf.copy_within((last_nl + 1)..old_idx, 0);
            self.idx = old_idx - last_nl - 1;
        }
    }
}

impl Write for BufLoggerInner {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.write_buf(s.as_bytes());
        Ok(())
    }
}

pub struct BufLogger {
    inner: NullLock<BufLoggerInner>,
}

impl BufLogger {
    pub const fn new() -> Self {
        Self {
            inner: NullLock::new(BufLoggerInner::new()),
        }
    }

    pub fn set_writer(&self, w: &'static (dyn LogWrite + Sync)) {
        self.inner.lock(|i| i.writer = Some(w))
    }

    pub fn log(&self, level: LogLevel, args: core::fmt::Arguments) {
        self.inner.lock(|i| {
            write!(i, "{}: {}", level, args).unwrap();
        })
    }

    pub fn write(&self, args: core::fmt::Arguments) {
        self.inner.lock(|i| i.write_fmt(args).unwrap())
    }

    pub fn flush(&self) {
        self.inner.lock(|i| i.flush())
    }
}
