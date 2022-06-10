use core::ptr::{read_volatile, write_volatile};

use crate::{print, trap::Frame, uart};

const PRIORITY: *mut u32 = 0xc000000 as *mut u32;
//const PENDING: *mut u32 = 0xc001000 as *mut u32;
const INT_ENABLE: *mut u32 = 0xc002080 as *mut u32;
const THRESHOLD: *mut u32 = 0xc201000 as *mut u32;
const CLAIM: *mut u32 = 0xc201004 as *mut u32;
const COMPLETE: *mut u32 = 0xc201004 as *mut u32;

// Use types to make sure the init order is correct.
// Also add disable method.
// It's becoming more of a Rust exercise than OS's.
pub fn init() {
    enable(QemuSource::Uart0);
    enable(QemuSource::Virtio8);
    set_priority(QemuSource::Uart0, 1);
    set_priority(QemuSource::Virtio8, 1);
    set_thresold(0);
    unsafe {
        riscv::register::sie::set_sext();
    }
}

pub fn handle_interrupts(_frame: &mut Frame) {
    if let Some(intr) = claim_intr() {
        use QemuSource::*;
        match intr.0 {
            Uart0 => match uart::get() as u8 {
                8 | 127 => print!("\x08 \x08"),
                b'\r' => print!("\r\n"),
                b => print!("{}", b as char),
            },

            Virtio8 => {
                //info!("virtio8");
            }

            // TODO: We can forget to handle Unknown variant,
            // causing the ClaimedSource to Drop with completing id 54.
            Unknown => unimplemented!("unkonwn plic interrupt"),

            _ => unimplemented!(),
        }
    }
}

#[derive(Debug)]
struct ClaimedSource(QemuSource);

impl Drop for ClaimedSource {
    fn drop(&mut self) {
        complete_intr(self.0);
    }
}

fn enable(intr: QemuSource) {
    let actual_id = 1 << intr as u32;
    unsafe {
        write_volatile(INT_ENABLE, read_volatile(INT_ENABLE) | actual_id);
    }
}

fn set_priority(intr: QemuSource, thres: u8) {
    let priority_reg = (PRIORITY as u32 + intr as u32 * 4) as *mut u32;
    unsafe {
        write_volatile(priority_reg, thres as u32);
    }
}

fn set_thresold(thres: u8) {
    unsafe {
        write_volatile(THRESHOLD, thres as u32);
    }
}

fn claim_intr() -> Option<ClaimedSource> {
    let intr = unsafe { read_volatile(CLAIM) };
    Some(ClaimedSource(QemuSource::from_u32(intr)?))
}

fn complete_intr(intr: QemuSource) {
    unsafe { write_volatile(COMPLETE, intr as u32) }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum QemuSource {
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
    fn from_u32(value: u32) -> Option<QemuSource> {
        use QemuSource::*;
        Some(match value {
            0 => return None,
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
        })
    }
}
