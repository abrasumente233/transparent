use core::alloc::{GlobalAlloc, Layout};

pub(crate) const PAGE_SIZE: usize = 4096;
pub(crate) const QEMU_MEMORY_BASE: usize = 0x80000000;
pub(crate) const QEMU_MEMORY_SIZE: usize = 0x08000000;

extern "C" {
    fn _kernel_end();
}

pub(crate) fn heap_start() -> usize {
    _kernel_end as usize
}

pub(crate) fn heap_end() -> usize {
    QEMU_MEMORY_BASE + QEMU_MEMORY_SIZE
}

use linked_list_allocator::LockedHeap;

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

/*
pub(crate) struct Dummy;

static mut ALLOCATED_PAGES: usize = 0xffff_ffff;

unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if ALLOCATED_PAGES == 0xffff_ffff {
            ALLOCATED_PAGES = heap_start() / PAGE_SIZE;
        }

        let pages = (layout.size() + PAGE_SIZE - 1) / PAGE_SIZE;
        let allocated = ALLOCATED_PAGES * PAGE_SIZE;
        ALLOCATED_PAGES += pages;
        allocated as *mut u8
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        
    }
}

#[global_allocator]
static ALLOCATOR: Dummy = Dummy;
*/

#[alloc_error_handler]
fn alloc_error_handler(layout: Layout) -> ! {
    panic!("allocation error: {:?}", layout);
}
