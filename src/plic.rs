use core::ptr::{read_volatile, write_volatile};

use crate::{print, println, trap::Frame, uart};

const PRIORITY: *mut u32 = 0xc000000 as *mut u32;
//const PENDING: *mut u32 = 0xc001000 as *mut u32;
const INT_ENABLE: *mut u32 = 0xc002080 as *mut u32;
const THRESHOLD: *mut u32 = 0xc201000 as *mut u32;
const CLAIM: *mut u32 = 0xc201004 as *mut u32;
const COMPLETE: *mut u32 = 0xc201004 as *mut u32;

static PLIC: Plic = Plic {};

// Use types to make sure the init order is correct.
// Also add disable method.
pub(crate) fn init() {
    PLIC.enable(QemuSource::Uart0)
        .set_priority(QemuSource::Uart0, 1)
        .set_thresold(0);
}

pub(crate) fn handle_interrupts(frame: &mut Frame) {
    let intr = PLIC.claim_intr();

    use QemuSource::*;
    match intr.0 {
        NoInterrupt => return,
        Uart0 => match uart::get() as u8 {
            8 | 127 => print!("\x08 \x08"),
            b'\r' => print!("\r\n"),
            b => print!("{}", b as char),
        },
        Unknown => {
            println!("Trap frame: {:?}", frame);
            panic!("Unknown interrupt: {:?}", intr);
        }
        _ => unimplemented!(),
    }
}

struct Plic {}

#[derive(Debug)]
struct ClaimedSource(QemuSource);

impl Drop for ClaimedSource {
    fn drop(&mut self) {
        if self.0 == QemuSource::NoInterrupt {
            return;
        }
        PLIC.complete_intr(self.0);
    }
}

impl Plic {
    fn enable(&self, intr: QemuSource) -> &Self {
        let actual_id = 1 << intr as u32;
        unsafe {
            write_volatile(INT_ENABLE, read_volatile(INT_ENABLE) | actual_id);
        }
        self
    }

    fn set_priority(&self, intr: QemuSource, thres: u8) -> &Self {
        let priority_reg = (PRIORITY as u32 + intr as u32 * 4) as *mut u32;
        unsafe {
            write_volatile(priority_reg, thres as u32);
        }
        self
    }

    fn set_thresold(&self, thres: u8) -> &Self {
        unsafe {
            write_volatile(THRESHOLD, thres as u32);
        }
        self
    }

    fn claim_intr(&self) -> ClaimedSource {
        let intr = unsafe { read_volatile(CLAIM) };
        ClaimedSource(QemuSource::from_u32(intr))
    }

    fn complete_intr(&self, intr: QemuSource) {
        unsafe { write_volatile(COMPLETE, intr as u32) }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq)]
enum QemuSource {
    NoInterrupt = 0,
    Virtio1 = 1,
    Virtio2 = 2,
    Virtio3 = 3,
    Virtio4 = 4,
    Virtio5 = 5,
    Virtio6 = 6,
    Virtio7 = 7,
    Virtio8 = 8,
    Uart0 = 10,
    Unknown = 54,
}

impl QemuSource {
    fn from_u32(value: u32) -> QemuSource {
        use QemuSource::*;
        match value {
            0 => NoInterrupt,
            1 => Virtio1,
            2 => Virtio2,
            3 => Virtio3,
            4 => Virtio4,
            5 => Virtio5,
            6 => Virtio6,
            7 => Virtio7,
            8 => Virtio8,
            10 => Uart0,
            _ => Unknown,
        }
    }
}