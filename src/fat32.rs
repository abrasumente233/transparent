#![allow(unused)]
use log::{info, debug};

use crate::block::BlockDevice;

pub struct Fat32<'a, B>
where
    B: BlockDevice,
{
    block: &'a mut B,
    bpb: BiosParameterBlockPacked,
}

impl<'a, B> Fat32<'a, B>
where
    B: BlockDevice,
{
    pub fn new(block: &'a mut B) -> Self {
        let mut buf = [0; 512];
        block.read(0, &mut buf).unwrap();
        let superblock = BiosParameterBlockPacked::from_bytes(&buf).unwrap();
        Fat32 { block, bpb: superblock }
    }

    pub fn check_fs(&self) {
        info!("{:?}", self.bpb);
        info!("{}", core::ptr::addr_of!(self.bpb.root_cluster) as usize - core::ptr::addr_of!(self.bpb) as usize);

        assert_eq!(self.bpb.jmp_boot()[0], 0xeb);
        assert_eq!(self.bpb.bytes_per_sector(), 512);
        assert_eq!(self.bpb.fats(), 2);
        assert_eq!(self.bpb.max_root_dir_entries(), 0);
        assert_eq!(self.bpb.total_sectors_16(), 0);
        assert_eq!(self.bpb.media(), 0xf8);
        assert_eq!(self.bpb.sectors_per_fat_16(), 0);
        assert_eq!(self.bpb.fat_version(), 0);
        assert_eq!(self.bpb.root_cluster(), 2);
        assert_eq!(self.bpb.fsinfo_sector(), 1);
        assert_eq!(self.bpb.backup_boot_sector(), 6);
        assert_eq!(self.bpb.sectors_per_cluster(), 1);

        info!("fat32 fs check passed");
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
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
    extended_flags: u16,
    fat_version: u16,
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

impl BiosParameterBlockPacked {
    fn from_bytes(bytes: &[u8]) -> Result<BiosParameterBlockPacked, &'static str> {
        if bytes.len() < 512 {
            return Err("not enough bytes");
        }
        let bpb: BiosParameterBlockPacked = unsafe { core::ptr::read(bytes.as_ptr() as *const Self) };
        Ok(bpb)
    }

    fn jmp_boot(&self) -> [u8; 3] {
        self.jmp_boot
    }

    fn oem_name(&self) -> [u8; 8] {
        self.oem_name
    }

    fn bytes_per_sector(&self) -> u16 {
        self.bytes_per_sector
    }

    fn sectors_per_cluster(&self) -> u8 {
        self.sectors_per_cluster
    }

    fn reserved_sectors(&self) -> u16 {
        self.reserved_sectors
    }

    fn fats(&self) -> u8 {
        self.fats
    }

    fn max_root_dir_entries(&self) -> u16 {
        self.max_root_dir_entries
    }

    fn total_sectors_16(&self) -> u16 {
        self.total_sectors_16
    }

    fn media(&self) -> u8 {
        self.media
    }

    fn sectors_per_fat_16(&self) -> u16 {
        self.sectors_per_fat_16
    }

    fn sectors_per_track(&self) -> u16 {
        self.sectors_per_track
    }

    fn heads(&self) -> u16 {
        self.heads
    }

    fn hidden_sectors(&self) -> u32 {
        self.hidden_sectors
    }

    fn total_sectors_32(&self) -> u32 {
        self.total_sectors_32
    }

    fn sectors_per_fat_32(&self) -> u32 {
        self.sectors_per_fat_32
    }

    fn extended_flags(&self) -> u16 {
        self.extended_flags
    }

    fn fat_version(&self) -> u16 {
        self.fat_version
    }

    fn root_cluster(&self) -> u32 {
        self.root_cluster
    }

    fn fsinfo_sector(&self) -> u16 {
        self.fsinfo_sector
    }

    fn backup_boot_sector(&self) -> u16 {
        self.backup_boot_sector
    }

    fn drive_number(&self) -> u8 {
        self.drive_number
    }

    fn boot_signature(&self) -> u8 {
        self.boot_signature
    }

    fn volume_id(&self) -> u32 {
        self.volume_id
    }

    fn volume_label(&self) -> [u8; 11] {
        self.volume_label
    }

    fn fat_type_label(&self) -> [u8; 8] {
        self.fat_type_label
    }
}
