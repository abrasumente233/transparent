use core::ptr::addr_of;

use crate::{
    print,
    virtio::{Device, DeviceStatus, Queue, PAGE_SIZE, VIRTIO_RING_SIZE},
};
use arrayvec::ArrayVec;
use bitflags::bitflags;
use lazy_static::lazy_static;

const BSIZE: usize = 512;

lazy_static! {
    static ref QUEUE: Queue = Queue::default();
}

pub(crate) fn init(dev: &'static mut Device) -> bool {
    // Reset the device.
    dev.status.write(DeviceStatus::RESET);

    // Set the ACKNOWLEDGE status bit: the guest OS has noticed the device.
    dev.status.write(DeviceStatus::ACKNOWLEDGE);

    // Set the DRIVER status bit: the guest OS knows how to drive the device.
    dev.status
        .write(DeviceStatus::ACKNOWLEDGE | DeviceStatus::DRIVER_OK);

    // Read device feature bits, and write the subset of feature bits understood by
    // the OS and driver to the device.
    // During this step the driver MAY read (but MUST NOT write)
    // the device-specific configuration fields to check that it can support the device before accepting it.
    let host_features = Feature::from_bits_truncate(dev.device_features.read());
    let ro = host_features.contains(Feature::RO);
    dev.driver_features.write(host_features.bits());

    // Set the FEATURES_OK status bit. The driver MUST NOT accept new feature bits after this step.
    dev.status
        .write(DeviceStatus::ACKNOWLEDGE | DeviceStatus::DRIVER_OK | DeviceStatus::FEATURES_OK);

    // Re-read device status to ensure the FEATURES_OK bit is still set: otherwise, the device does not support our subset of features and the device is unusable.
    let features_ok = dev.status.read().contains(DeviceStatus::FEATURES_OK);
    if !features_ok {
        print!("features failed...");
        dev.status.write(DeviceStatus::FAILED);
        return false;
    }

    // Perform device-specific setup, including discovery of virtqueues for the device,
    // optional per-bus setup, reading and possibly writing the device’s virtio configuration space,
    // and population of virtqueues.

    // Configure virtqueue

    // Select the queue writing its index (first queue is 0) to QueueSel.
    dev.queue_sel.write(0);

    // Check if the queue is not already in use: read QueueReady, and expect a returned value of zero (0x0).
    let queue_ready = dev.queue_ready.read();
    if queue_ready != 0 {
        print!("queue already in use...");
        return false;
    }

    // Read maximum queue size (number of elements) from QueueNumMax. If the returned value is zero (0x0) the queue is not available.

    let qnmax = dev.queue_num_max.read();
    if qnmax == 0 {
        print!("queue not available...");
        return false;
    } else if VIRTIO_RING_SIZE > qnmax as usize {
        print!("virtio ring size too big...");
        return false;
    }

    // Allocate and zero the queue memory, making sure the memory is physically contiguous.
    // NOTE: We store the queue as a global for now.

    // Notify the device about the queue size by writing the size to QueueNum.
    dev.queue_num.write(VIRTIO_RING_SIZE as u32);

    // Write physical addresses of the queue’s Descriptor Area,
    // Driver Area and Device Area to (respectively) the QueueDescLow/QueueDescHigh,
    // QueueDriverLow/QueueDriverHigh and QueueDeviceLow/QueueDeviceHigh register pairs.
    dev.guest_page_size.write(PAGE_SIZE as u32);
    let queue_addr = addr_of!(QUEUE) as *mut Queue;
    dev.queue_pfn
        .write((queue_addr as usize / PAGE_SIZE) as u32);

    // Set the DRIVER_OK status bit. At this point the device is “live”.
    dev.status
        .write(DeviceStatus::ACKNOWLEDGE | DeviceStatus::FEATURES_OK | DeviceStatus::DRIVER_OK);

    unsafe {
        BLOCK_DEVICES.push(BlockDevice {
            queue: queue_addr as *mut Queue,
            device: dev,
            idx: 0,
            ack_used_idx: 0,
            read_only: ro,
        });
    }

    true
}

// TODO: Make it Rusty
pub(crate) fn block_op(dev: usize, buffer: *mut u8, size: usize, offset: usize, write: bool) -> bool {
    unsafe {
        let bdev = BLOCK_DEVICES.get(dev).unwrap();
        if bdev.read_only && write {
            return false;
        }

        let sector = offset / BSIZE;
    }
    true
}

static mut BLOCK_DEVICES: ArrayVec<BlockDevice, 8> = ArrayVec::new_const();

pub(crate) struct BlockDevice<'a> {
    pub(crate) queue: *mut Queue,
    pub(crate) device: &'a mut Device,
    pub(crate) idx: u16,
    pub(crate) ack_used_idx: u16,
    pub(crate) read_only: bool,
}

bitflags! {
    pub(crate) struct Feature: u32 {
        /// Maximum size of any single segment is in size_max.
        const SIZE_MAX = 1 << 1;
        /// Maximum number of segments in a request is in seg_max.
        const SEG_MAX = 1 << 2;
        /// Disk-style geometry specified in geometry.
        const GEOMETRY = 1 << 4;
        /// Device is read-only.
        const RO = 1 << 5;
        /// Block size of disk is in blk_size.
        const BLK_SIZE = 1 << 6;
        /// Cache flush command support.
        const FLUSH = 1 << 9;
        /// Device exports information on optimal I/O alignment.
        const TOPOLOGY = 1 << 10;
        /// Device can toggle its cache between writeback and writethrough modes.
        const CONFIG_WCE = 1 << 11;
        /// Device can support discard command, maximum discard sectors size in max_discard_sectors and maximum discard segment number in max_discard_seg.
        const DISCARD = 1 << 13;
        /// Device can support write zeroes command, maximum write zeroes sectors size in max_write_zeroes_sectors and maximum write zeroes segment number in max_write_zeroes_seg.
        const WRITE_ZEROES = 1 << 14;
    }
}
