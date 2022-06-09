use alloc::alloc::{alloc, dealloc, Layout};

type VirtAddr = usize;
type PhysAddr = usize;

#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let layout = Layout::from_size_align(0x1000 * pages, 0x1000).unwrap();
    unsafe { alloc(layout) as PhysAddr }
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(paddr: PhysAddr, pages: usize) -> i32 {
    //println!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
    let layout = Layout::from_size_align(0x1000 * pages, 0x1000).unwrap();
    unsafe {
        dealloc(paddr as *mut u8, layout);
    }
    0
}

// NOTE: We haven't enabled virtual memory yet.
#[no_mangle]
extern "C" fn virtio_phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    paddr
}

#[no_mangle]
extern "C" fn virtio_virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
    vaddr
}
