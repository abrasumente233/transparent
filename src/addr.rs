use core::ops::{Index, IndexMut};

use bit_field::BitField;
use bitflags::bitflags;

/// A passed `u64` was not a valid virtual address.
///
/// This means that bits 39 to 64 are not
/// a valid sign extension and are not null either. So automatic sign extension would have
/// overwritten possibly meaningful bits. This likely indicates a bug, for example an invalid
/// address calculation.
///
/// Contains the invalid address.
pub struct VirtAddrNotValid(pub u64);

impl core::fmt::Debug for VirtAddrNotValid {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple("VirtAddrNotValid")
            .field(&format_args!("{:#x}", self.0))
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct VirtAddr(u64);

impl VirtAddr {
    /// Creates a new canonical virtual address.
    ///
    /// This function performs sign extension of bit 38 to make the address canonical.
    ///
    /// # Panics
    /// This function panics if the bits in the range 39 to 64 contain data (i.e. are not null and no sign extension).
    #[inline]
    fn new(addr: u64) -> VirtAddr {
        Self::try_new(addr).expect(
            "address passed to VirtAddr::new must not contain any data \
             in bits 39 to 64",
        )
    }

    /// Tries to create a new canonical virtual address.
    ///
    /// This function tries to performs sign
    /// extension of bit 38 to make the address canonical. It succeeds if bits 39 to 64 are
    /// either a correct sign extension (i.e. copies of bit 38) or all null. Else, an error
    /// is returned.
    #[inline]
    pub fn try_new(addr: u64) -> Result<VirtAddr, VirtAddrNotValid> {
        match addr.get_bits(38..64) {
            0 | 0x1ffff => Ok(VirtAddr(addr)),     // address is canonical
            1 => Ok(VirtAddr::new_truncate(addr)), // address needs sign extension
            _ => Err(VirtAddrNotValid(addr)),
        }
    }

    /// Creates a new canonical virtual address, throwing out bits 39..64.
    ///
    /// This function performs sign extension of bit 38 to make the address canonical, so
    /// bits 39 to 64 are overwritten. If you want to check that these bits contain no data,
    /// use `new` or `try_new`.
    #[inline]
    pub const fn new_truncate(addr: u64) -> VirtAddr {
        // By doing the right shift as a signed operation (on a i64), it will
        // sign extend the value, repeating the leftmost bit.
        VirtAddr(((addr << 25) as i64 >> 25) as u64)
    }

    /// Creates a new virtual address, without any checks.
    ///
    /// ## Safety
    ///
    /// You must make sure bits 39..64 are equal to bit 38. This is not checked.
    #[inline]
    pub const unsafe fn new_unsafe(addr: u64) -> VirtAddr {
        VirtAddr(addr)
    }

    /// Converts the address to an `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the 12-bit page offset of this virtual address
    #[inline]
    pub fn page_offset(self) -> u64 {
        self.0.get_bits(0..12)
    }

    /// Returns the 9-bit VPN2 of this virtual address
    #[inline]
    pub fn vpn2(self) -> u16 {
        self.0.get_bits(30..39) as u16
    }

    /// Returns the 9-bit VPN1 of this virtual address
    #[inline]
    pub fn vpn1(self) -> u16 {
        self.0.get_bits(21..30) as u16
    }

    /// Returns the 9-bit VPN0 of this virtual address
    #[inline]
    pub fn vpn0(self) -> u16 {
        self.0.get_bits(12..21) as u16
    }
}

pub struct PhysAddr(u64);

impl PhysAddr {
    /// Creates a new physical address.
    #[inline]
    pub const fn new(addr: u64) -> PhysAddr {
        PhysAddr(addr)
    }

    /// Converts the address to an `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug)]
    pub struct PageTableFlags: u64 {
        const VALID = 1 << 0;
        const READABLE = 1 << 1;
        const WRITABLE = 1 << 2;
        const EXECUTABLE = 1 << 3;
        const USER_ACCESSIBLE = 1 << 4;
        const GLOBAL = 1 << 5;
        const ACCESSED = 1 << 6;
        const DIRTY = 1 << 7;
    }
}

impl PageTableFlags {
    #[inline]
    pub const fn is_valid(self) -> bool {
        self.contains(PageTableFlags::VALID)
    }

    #[inline]
    pub const fn is_leaf(self) -> bool {
        self.is_valid()
            && (self.contains(PageTableFlags::READABLE)
                || self.contains(PageTableFlags::WRITABLE)
                || self.contains(PageTableFlags::EXECUTABLE))
    }
}

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PTEntry {
    entry: u64,
}

impl core::fmt::Debug for PTEntry {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("PTEntry")
            .field("frame", &format_args!("{:#x}", self.addr().as_u64()))
            .field("flags", &self.flags())
            .finish()
    }
}

impl PTEntry {
    /// Creates an unused page table entry.
    #[inline]
    pub const fn new() -> Self {
        PTEntry { entry: 0 }
    }

    /// Map the entry to the specified physical address with the specified flags.
    #[inline]
    pub fn set(&mut self, addr: PhysAddr, flags: PageTableFlags) {
        // FIXME: assert that addr is page aligned
        self.entry = (addr.as_u64() >> 2) | flags.bits();
    }

    /// Returns the physical address mapped by this entry, might be zero.
    #[inline]
    pub fn addr(&self) -> PhysAddr {
        // The address is stored in bits 53..10
        PhysAddr::new(self.entry.get_bits(10..54) << 12)
    }

    /// Returns the flags of this entry.
    #[inline]
    pub const fn flags(&self) -> PageTableFlags {
        PageTableFlags::from_bits_truncate(self.entry)
    }

    /// Sets the flags of this entry.
    #[inline]
    pub fn set_flags(&mut self, flags: PageTableFlags) {
        self.entry = self.addr().as_u64() | flags.bits();
    }

    #[inline]
    pub fn set_unused(&mut self) {
        self.set_flags(PageTableFlags::empty());
    }

    #[inline]
    pub const fn as_u64(&self) -> u64 {
        self.entry
    }
}

#[repr(align(4096))]
#[repr(C)]
#[derive(Clone)]
pub struct PageTable {
    entries: [PTEntry; 512],
}

impl PageTable {
    /// Creates an empty page table.
    #[inline]
    pub const fn new() -> Self {
        const EMPTY: PTEntry = PTEntry::new();
        PageTable {
            entries: [EMPTY; 512],
        }
    }

    /// Clears all entries.
    #[inline]
    pub fn zero(&mut self) {
        for entry in self.entries.iter_mut() {
            entry.set_unused();
        }
    }

    /// Returns an iterator over the entries of the page table.
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &PTEntry> {
        self.entries.iter()
    }

    /// Returns an iterator that allows modifying the entries of the page table.
    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut PTEntry> {
        self.entries.iter_mut()
    }
}

impl Index<u16> for PageTable {
    type Output = PTEntry;

    #[inline]
    fn index(&self, index: u16) -> &Self::Output {
        &self.entries[index as usize]
    }
}

impl IndexMut<u16> for PageTable {
    #[inline]
    fn index_mut(&mut self, index: u16) -> &mut Self::Output {
        &mut self.entries[index as usize]
    }
}
