#![allow(unused)]
use log::info;

use crate::block::BlockDevice;

pub struct Fat32<'a, B>
where
    B: BlockDevice,
{
    block: &'a mut B,
    bpb: BiosParameterBlock,
}

impl<'a, B> Fat32<'a, B>
where
    B: BlockDevice,
{
    /*
    pub fn new(block: &'a mut B) -> Self {
        let mut buf = [0; 512];
        block.read(0, &mut buf).unwrap();
        let superblock = BiosParameterBlock::from_bytes(&buf).unwrap();
        Fat32 { block, bpb: superblock }
    }
    */

    pub fn check_fs(&self) {
        assert_eq!(self.bpb.jmp_boot[0], 0xeb);
        assert_eq!(self.bpb.bytes_per_sector, 512);
        info!("fat32 fs check passed");
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(packed)]
struct BiosParameterBlockPacked {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    max_root_dir_entries: u16,
    total_sectors_16: u16,
    media: u8,
    sectors_per_fat_16: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    sectors_per_fat_32: u32,
    extended_flags: u32,
    fat_version: u32,
    root_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    _reserved_0: [u8; 12],
    drive_number: u8,
    _reserved_1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fat_type_label: [u8; 8],
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
struct BiosParameterBlock {
    jmp_boot: [u8; 3],
    oem_name: [u8; 8],
    bytes_per_sector: u16,
    sectors_per_cluster: u8,
    reserved_sectors: u16,
    fats: u8,
    max_root_dir_entries: u16,
    total_sectors_16: u16,
    media: u8,
    sectors_per_fat_16: u16,
    sectors_per_track: u16,
    heads: u16,
    hidden_sectors: u32,
    total_sectors_32: u32,
    sectors_per_fat_32: u32,
    extended_flags: u32,
    fat_version: u32,
    root_cluster: u32,
    fsinfo_sector: u16,
    backup_boot_sector: u16,
    _reserved_0: [u8; 12],
    drive_number: u8,
    _reserved_1: u8,
    boot_signature: u8,
    volume_id: u32,
    volume_label: [u8; 11],
    fat_type_label: [u8; 8],
}

impl BiosParameterBlock {
    fn from_bytes(bytes: &[u8]) -> Result<(), &'static str> {
        if bytes.len() < 512 {
            return Err("not enough bytes");
        }
        //let bpb: BiosParameterBlockPacked = unsafe { core::ptr::read(bytes.as_ptr() as *const Self) };
        //Ok(bpb.clone())
        Ok(())
    }
}
