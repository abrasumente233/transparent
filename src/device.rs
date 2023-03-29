use device_tree::{util::SliceRead, DeviceTree, Node};
use log::trace;
use virtio_drivers::{DeviceType, VirtIOBlk, VirtIOHeader};

use crate::block::{VirtioBlock, BLK};

pub fn init(device_tree_addr: usize) {
    init_device_tree(device_tree_addr);
}

fn init_device_tree(dtb: usize) {
    trace!("device tree @ {:#x}", dtb);
    #[repr(C)]
    struct DtbHeader {
        be_magic: u32,
        be_size: u32,
    }
    let header = unsafe { &*(dtb as *const DtbHeader) };
    let magic = u32::from_be(header.be_magic);
    const DEVICE_TREE_MAGIC: u32 = 0xd00dfeed;
    assert_eq!(magic, DEVICE_TREE_MAGIC);
    let size = u32::from_be(header.be_size);
    let dtb_data = unsafe { core::slice::from_raw_parts(dtb as *const u8, size as usize) };
    let dt = DeviceTree::load(dtb_data).expect("failed to parse device tree");
    walk_dt_node(&dt.root);
}

fn walk_dt_node(dt: &Node) {
    if let Ok(compatible) = dt.prop_str("compatible") {
        if compatible == "virtio,mmio" {
            virtio_probe(dt);
        }
    }
    if let Ok(device_type) = dt.prop_str("device_type") {
        if device_type == "memory" {
            memory_probe(dt)
        }
    }
    for child in dt.children.iter() {
        walk_dt_node(child);
    }
}

// TODO: call once
fn memory_probe(db: &Node) {
    if let Some(reg) = db.prop_raw("reg") {
        // TODO: There could be mutiple ranges of memory
        let start = reg.as_slice().read_be_u64(0).unwrap();
        let length = reg.as_slice().read_be_u64(8).unwrap();
        log::info!("memory range: {:#x} ~ {:#x}", start, start + length);
    }
}

fn virtio_probe(node: &Node) {
    if let Some(reg) = node.prop_raw("reg") {
        let paddr = reg.as_slice().read_be_u64(0).unwrap();
        let size = reg.as_slice().read_be_u64(8).unwrap();
        let vaddr = paddr;
        trace!("walk dt addr={:#x}, size={:#x}", paddr, size);
        let header = unsafe { &mut *(vaddr as *mut VirtIOHeader) };
        trace!(
            "Detected virtio device with vendor id {:#X}",
            header.vendor_id()
        );
        trace!("Device tree node {:?}", node);
        match header.device_type() {
            DeviceType::Block => virtio_blk(header),
            t => trace!("Unrecognized virtio device: {:?}", t),
        }
    }
}

fn virtio_blk(header: &'static mut VirtIOHeader) {
    let blk = VirtIOBlk::new(header).expect("failed to create blk driver");
    /*
    let mut input = [0xffu8; 512];
    let mut output = [0u8; 512];
    for i in 0..32 {
        for x in input.iter_mut() {
            *x = i as u8;
        }
        blk.write_block(i, &input).expect("failed to write block");
        blk.read_block(i, &mut output).expect("failed to read");
        assert_eq!(input, output);
    }
    info!("virtio-blk test finished");
    */
    unsafe {
        BLK = Some(VirtioBlock(blk));
    }
}
