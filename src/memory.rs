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

/// Translates the given virtual address to the mapped physical address, or
/// `None` if the address is not mapped.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is identity mapped.
pub unsafe fn translate_addr(addr: VirtAddr) -> Option<PhysAddr> {
    translate_addr_inner(addr)
}

/// Private function that is called by `translate_addr`.
///
/// This function is safe to limit the scope of `unsafe` because Rust treats
/// the whole body of unsafe functions as an unsafe block. This function must
/// only be reachable through `unsafe fn` from outside of this module.
fn translate_addr_inner(addr: VirtAddr) -> Option<PhysAddr> {
    // read the active level 4 frame from the CR3 register
    let level_3_table_frame = (satp::read().ppn() << 12) as u64;

    let table_indexes = [addr.vpn2(), addr.vpn1(), addr.vpn0()];
    let mut frame = level_3_table_frame;

    // traverse the multi-level page table
    for (i, &index) in table_indexes.iter().enumerate() {
        // convert the frame into a page table reference
        let virt = frame; // because we identity mapped
        let table_ptr: *const PageTable = virt as *const PageTable;
        let table = unsafe { &*table_ptr };

        // read the page table entry and update `frame`
        let entry = &table[index];
        if !entry.is_valid() {
            return None;
        }
        frame = entry.addr().as_u64();
        if entry.flags().is_leaf() {
            frame += match i {
                0 => ((addr.vpn1() as u64) << 21) | ((addr.vpn0() as u64) << 12),
                1 => ((addr.vpn0() as u64) << 12),
                _ => 0,
            };
            break;
        }
    }

    // calculate the physical address by adding the page offset
    Some(PhysAddr::new(frame + u64::from(addr.page_offset())))
}
