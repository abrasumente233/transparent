// NOTE: We support only SV-39 now.

use riscv::{asm::sfence_vma_all, register::satp};

use crate::addr::{PTEntry, PageTable, PageTableFlags, PhysAddr, VirtAddr};

static mut ROOT_PAGE_TABLE: PageTable = PageTable::new();

pub fn init() {
    // Identity map the a 1 GiB page to whole physical memory
    let virt_base = VirtAddr::new_truncate(0x80000000);

    let mut ident_pte = PTEntry::new();
    ident_pte.set(
        PhysAddr::new(0x80000000), // FIXME: Hardcoded for now for QEMU
        PageTableFlags::VALID
            | PageTableFlags::READABLE
            | PageTableFlags::WRITABLE
            | PageTableFlags::EXECUTABLE,
    );

    // FIXME: We have enabled virtual memory, but at what cost?
    // We also have to identity map device memory...
    let mut device_pte = PTEntry::new();
    device_pte.set(
        PhysAddr::new(0),
        PageTableFlags::VALID | PageTableFlags::READABLE | PageTableFlags::WRITABLE,
    );

    unsafe {
        ROOT_PAGE_TABLE[virt_base.vpn2()] = ident_pte;
        ROOT_PAGE_TABLE[0] = device_pte;
    }

    // enable virtual memory
    let root_table_addr = unsafe { &ROOT_PAGE_TABLE as *const _ as usize };
    unsafe {
        satp::set(satp::Mode::Sv39, 0 /* TODO */, root_table_addr >> 12);
        sfence_vma_all();
    }
}

pub unsafe fn active_level_3_table() -> &'static mut PageTable {
    let phys = satp::read().ppn() << 12;

    // Since we have identity mapped, virt is also phys
    let virt = phys;

    let page_table_ptr: *mut PageTable = virt as *mut PageTable;
    &mut *page_table_ptr
}