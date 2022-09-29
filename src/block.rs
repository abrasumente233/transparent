use core::ops::Deref;

use virtio_drivers::VirtIOBlk;

pub static mut BLK: Option<VirtioBlock> = None;

// TODO: Implement a copyless BlockDevice.
// Because we could repetitively read the same block, which can
// result in wasted I/O or extra copies even with buffering.
pub trait BlockDevice {
    const BSIZE: usize = 512;

    /// Read a block into `buf`
    fn read(&mut self, blk_id: usize, buf: &mut [u8]) -> Option<usize>;

    /// Write `buf` to a block
    fn write(&mut self, blk_id: usize, buf: &[u8]) -> Option<usize>;
}

pub struct VirtioBlock<'a>(pub VirtIOBlk<'a>);

impl<'a> Deref for VirtioBlock<'a> {
    type Target = VirtIOBlk<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> BlockDevice for VirtioBlock<'a> {
    fn read(&mut self, blk_id: usize, buf: &mut [u8]) -> Option<usize> {
        self.0.read_block(blk_id, buf).map(|_| buf.len()).ok()
    }

    fn write(&mut self, blk_id: usize, buf: &[u8]) -> Option<usize> {
        self.0.write_block(blk_id, buf).map(|_| buf.len()).ok()
    }
}
