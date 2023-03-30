// NOTE: We support only SV-39 now.
// FIXME: Add tests

use riscv::{asm::sfence_vma_all, register::satp};

use crate::addr::{PageTable, PageTableFlags, PhysAddr, VirtAddr};

static mut ROOT_PAGE_TABLE: PageTable = PageTable::new();

pub fn init() {
    let root_table_addr = unsafe { &ROOT_PAGE_TABLE as *const _ as u64 };

    // Identity map the a 1 GiB page to whole physical memory
    unsafe {
        map_to_with_pt(
            root_table_addr,
            VirtAddr::new(0x80000000),
            PhysAddr::new(0x80000000),
            PageTableFlags::VRWX,
            0,
            &mut FrameAllocator,
        )
    }
    .expect("map failed");

    // FIXME: We have enabled virtual memory, but at what cost?
    // We also have to identity map device memory...
    unsafe {
        map_to_with_pt(
            root_table_addr,
            VirtAddr::new(0),
            PhysAddr::new(0),
            PageTableFlags::VRW,
            0,
            &mut FrameAllocator,
        )
    }
    .expect("map failed");

    // enable virtual memory
    unsafe {
        satp::set(
            satp::Mode::Sv39,
            0, /* TODO */
            (root_table_addr >> 12) as usize,
        );
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
    Some(PhysAddr::new(frame + addr.page_offset()))
}

/// An allocator that allocates 4KiB physical frames.
pub struct FrameAllocator;

impl FrameAllocator {
    pub fn new() -> Self {
        Self {}
    }

    pub fn alloc_frame(&mut self) -> Option<PhysAddr> {
        None
    }

    pub fn dealloc_frame(&mut self, _frame: PhysAddr) {}
}

#[derive(Debug)]
pub enum MapToError {
    /// An additional frame was needed for the mapping process, but the frame allocator returned None.
    FrameAllocationFailed,

    /// An upper level page table entry has the HUGE_PAGE flag set,
    /// which means that the given page is part of an already mapped huge page.
    ParentEntryHugePage,

    /// The given page is already mapped to a physical frame.
    PageAlreadyMapped(PhysAddr),
}

pub unsafe fn map_to(
    page: VirtAddr,
    frame: PhysAddr,
    flags: PageTableFlags,
    level: u8,
    frame_allocator: &mut FrameAllocator,
) -> Result<(), MapToError> {
    // read the active level 4 frame from the CR3 register
    let level_3_table_frame = (satp::read().ppn() << 12) as u64;

    map_to_with_pt(
        level_3_table_frame,
        page,
        frame,
        flags,
        level,
        frame_allocator,
    )
}

/// Maps the given virtual page to the given physical frame with the given
/// flags.
///
/// # Safety
///
/// Creating page table mappings is a fundamentally unsafe operation because
/// there are various ways to break memory safety through it. For example,
/// re-mapping an in-use page to a different frame changes and invalidates all
/// values stored in that page, resulting in undefined behavior on the next use.
///
/// Apart from that, for example, when `level == 0`, `VirtAddr` and `PhysAddr`'s PN[1]
/// and PN[0] must be 0.
pub unsafe fn map_to_with_pt(
    pt_frame: u64,
    page: VirtAddr,
    frame: PhysAddr,
    flags: PageTableFlags,
    level: u8,
    frame_allocator: &mut FrameAllocator,
) -> Result<(), MapToError> {
    let table_indexes = [page.vpn2(), page.vpn1(), page.vpn0()];

    let mut pt = pt_frame;

    for l in 0..=level {
        let index = table_indexes[l as usize];
        let virt = pt; // because we identity mapped
        let table_ptr: *mut PageTable = virt as *mut PageTable;
        let table = &mut *table_ptr;
        let entry = &mut table[index];

        if l == level {
            if entry.is_valid() {
                if level == 2 {
                    return Err(MapToError::PageAlreadyMapped(entry.addr()));
                } else {
                    return Err(MapToError::ParentEntryHugePage);
                }
            } else {
                entry.set(frame, flags);
                return Ok(());
            }
        }

        // If we are not at the last level, and the entry is not valid, meaning
        // it is not pointing to a page table, we need to allocate a new page.
        if !entry.is_valid() {
            // allocate a new frame
            let frame = match frame_allocator.alloc_frame() {
                Some(frame) => frame,
                None => return Err(MapToError::FrameAllocationFailed),
            };
            entry.set(frame, PageTableFlags::VALID);
        }

        pt = entry.addr().as_u64();
    }

    // FIXME: Return a `MapperFlush` helper that can be used to flush the TLB.
    Ok(())
}
