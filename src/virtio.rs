use core::mem::size_of;

use bitflags::bitflags;
use volatile::{ReadOnly, ReadWrite, WriteOnly};

use crate::{block, print, println};

#[allow(dead_code)]
pub(crate) struct Device {
    pub(crate) magic: ReadOnly<u32>,
    pub(crate) version: ReadOnly<u32>,
    pub(crate) device_id: ReadOnly<DeviceType>,
    pub(crate) vendor_id: ReadOnly<u32>,
    pub(crate) device_features: ReadOnly<u32>,
    pub(crate) device_features_sel: WriteOnly<u32>,
    __r1: [ReadOnly<u32>; 2],
    pub(crate) driver_features: WriteOnly<u32>,
    pub(crate) driver_features_sel: WriteOnly<u32>,
    pub(crate) guest_page_size: WriteOnly<u32>,
    __r2: ReadOnly<u32>,
    pub(crate) queue_sel: WriteOnly<u32>,
    pub(crate) queue_num_max: ReadOnly<u32>,
    pub(crate) queue_num: WriteOnly<u32>,
    pub(crate) queue_align: WriteOnly<u32>,
    pub(crate) queue_pfn: ReadWrite<u32>,
    pub(crate) queue_ready: ReadWrite<u32>,
    __r3: [ReadOnly<u32>; 2],
    pub(crate) queue_notify: WriteOnly<u32>,
    __r4: [ReadOnly<u32>; 3],
    pub(crate) interrupt_status: ReadOnly<u32>,
    pub(crate) interrupt_ack: WriteOnly<u32>,
    __r5: [ReadOnly<u32>; 2],
    pub(crate) status: ReadWrite<DeviceStatus>,
    __r6: [ReadOnly<u32>; 3],
    pub(crate) queue_desc_low: WriteOnly<u32>,
    pub(crate) queue_desc_high: WriteOnly<u32>,
    __r7: [ReadOnly<u32>; 2],
    pub(crate) queue_avail_low: WriteOnly<u32>,
    pub(crate) queue_avail_high: WriteOnly<u32>,
    __r8: [ReadOnly<u32>; 2],
    pub(crate) queue_used_low: WriteOnly<u32>,
    pub(crate) queue_used_high: WriteOnly<u32>,
    __r9: [ReadOnly<u32>; 21],
    pub(crate) config_generation: ReadOnly<u32>,
}

impl Device {
    unsafe fn from_addr<'a, T>(addr: *mut T) -> &'a mut Self {
        &mut *(addr as *mut Self)
    }
}

bitflags! {
    pub(crate) struct DeviceStatus: u32 {
        /// Reset the device.
        const RESET = 0;

        /// Indicates that the guest OS has found the device and
        /// recognized it as a valid virtio device.
        const ACKNOWLEDGE = 1;

        /// Indicates that the guest OS knows how to drive the device.
        /// Note: There could be a significant (or infinite) delay before setting this bit.
        /// For example, under Linux, drivers can be loadable modules.
        const DRIVER = 2;

        /// Indicates that something went wrong in the guest,
        /// and it has given up on the device.
        /// This could be an internal error, or the driver didn’t like the device for some reason,
        /// or even a fatal error during device operation.
        const FAILED = 128;

        /// Indicates that the driver has acknowledged all the features it understands,
        /// and feature negotiation is complete.
        const FEATURES_OK = 8;

        /// Indicates that the driver is set up and ready to drive the device.
        const DRIVER_OK = 4;

        /// Indicates that the device has experienced an error from which it can’t recover.
        const DEVICE_NEED_RESET = 64;
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub(crate) enum DeviceType {
    Reserved = 0,
    NetworkCard = 1,
    Block = 2,
    Console = 3,
    EntropySource = 4,
    MemoryBallooning = 5,
    IOMemory = 6,
    Rpmsg = 7,
    SCSIHost = 8,
    Transport = 9,
    Mac80211Wlan = 10,
    RprocSerial = 11,
    VirtioCAIF = 12,
    MemoryBalloon = 13,
    GPU = 16,
    Timer = 17,
    Input = 18,
    Socket = 19,
    Crypto = 20,
    SignalDistributionModule = 21,
    Pstore = 22,
    IOMMU = 23,
    Memory = 24,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct Desc {
    pub(crate) addr: u64,
    pub(crate) len: u32,
    pub(crate) flags: u16,
    pub(crate) next: u16,
}

#[repr(C)]
pub(crate) struct Avail {
    pub(crate) flags: u16,
    pub(crate) idx: u16,
    pub(crate) ring: [u16; VIRTIO_RING_SIZE],
    pub(crate) event: u16,
}

#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub(crate) struct UsedElem {
    pub(crate) id: u32,
    pub(crate) len: u32,
}

#[repr(C)]
pub(crate) struct Used {
    pub flags: u16,
    pub idx: u16,
    pub ring: [UsedElem; VIRTIO_RING_SIZE],
    pub event: u16,
}

#[repr(C)]
pub(crate) struct Queue {
    pub(crate) desc: [Desc; VIRTIO_RING_SIZE],
    pub(crate) avail: Avail,
    pub _pad: [u8; PAGE_SIZE - size_of::<Desc>() * VIRTIO_RING_SIZE - size_of::<Avail>()],
    pub(crate) used: Used,
}

impl Default for Queue {
    fn default() -> Self {
        Self {
            desc: [Desc {
                addr: 0,
                len: 0,
                flags: 0,
                next: 0,
            }; VIRTIO_RING_SIZE],
            avail: Avail {
                flags: 0,
                idx: 0,
                ring: [0; VIRTIO_RING_SIZE],
                event: 0,
            },
            _pad: [0; PAGE_SIZE - size_of::<Desc>() * VIRTIO_RING_SIZE - size_of::<Avail>()],
            used: Used {
                flags: 0,
                idx: 0,
                ring: [UsedElem { id: 0, len: 0 }; VIRTIO_RING_SIZE],
                event: 0,
            },
        }
    }
}

// According to the documentation, this must be a power
// of 2 for the new style. So, I'm changing this to use
// 1 << instead because that will enforce this standard.
pub const PAGE_SIZE: usize = 4096;

const VIRTIO_MAGIC: u32 = 0x74726976;
const VIRTIO_START: usize = 0x1000_1000;
const VIRTIO_END: usize = 0x1000_9000;
const VIRTIO_STRIDE: usize = 0x1000;
pub const VIRTIO_RING_SIZE: usize = 1 << 7;

pub(crate) fn probe_qemu() {
    for addr in (VIRTIO_START..VIRTIO_END).step_by(VIRTIO_STRIDE) {
        print!("probing virtio probing at 0x{:x}... ", addr);

        let device = unsafe { Device::from_addr(addr as *mut usize) };
        if device.magic.read() != VIRTIO_MAGIC {
            println!("not virtio");
        } else if device.device_id.read() == DeviceType::Reserved {
            println!("not connected");
        } else {
            print!("version {}. ", device.version.read());
            match device.device_id.read() {
                DeviceType::Block => {
                    print!("block device. ");
                    if block::init(device) {
                        println!("setup ok");
                    } else {
                        println!("setup failed");
                    }
                }
                _ => println!("unkonwn device"),
            }
        }
    }
}
