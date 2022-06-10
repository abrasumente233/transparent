use alloc::alloc::{alloc, dealloc, Layout};
use linked_list_allocator::LockedHeap;

use crate::align::{Aligned, A4096};

pub const PAGE_SIZE: usize = 4096;
//pub const QEMU_MEMORY_BASE: usize = 0x80000000;
//pub const QEMU_MEMORY_SIZE: usize = 0x08000000;
pub const HEAP_SIZE: usize = 0x04000000;

static HEAP: Aligned<A4096, [u8; HEAP_SIZE]> = Aligned([0; HEAP_SIZE]);

extern "C" {
    fn _kernel_end();
}

pub fn heap_start() -> usize {
    //_kernel_end as usize
    HEAP.as_ptr() as usize
}

pub fn heap_end() -> usize {
    //QEMU_MEMORY_BASE + QEMU_MEMORY_SIZE
    heap_start() + HEAP_SIZE
}

#[global_allocator]
static ALLOCATOR: LockedHeap = LockedHeap::empty();

pub fn init() {
    let heap_start = heap_start();
    let heap_end = heap_end();
    let heap_size = heap_end - heap_start;
    unsafe {
        ALLOCATOR.lock().init(heap_start, heap_size);
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}

// VirtIO allocation interfaces

type VirtAddr = usize;
type PhysAddr = usize;

#[no_mangle]
extern "C" fn virtio_dma_alloc(pages: usize) -> PhysAddr {
    let layout = Layout::from_size_align(PAGE_SIZE * pages, PAGE_SIZE).unwrap();
    unsafe { alloc(layout) as PhysAddr }
}

#[no_mangle]
extern "C" fn virtio_dma_dealloc(paddr: PhysAddr, pages: usize) -> i32 {
    //println!("dealloc DMA: paddr={:#x}, pages={}", paddr, pages);
    let layout = Layout::from_size_align(PAGE_SIZE * pages, PAGE_SIZE).unwrap();
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
