#![allow(unused)]
use log::{debug, info};

use crate::{block::BlockDevice, println};

pub struct Fat32<'b, B>
where
    B: BlockDevice,
{
    block: &'b mut B,
    bpb: BiosParameterBlockPacked,
}

impl<'b, B> Fat32<'b, B>
where
    B: BlockDevice,
{
    pub fn new(block: &'b mut B) -> Self {
        let mut buf = [0; 512];
        block.read(0, &mut buf).unwrap();
        assert_eq!(buf[510], 0x55);
        assert_eq!(buf[511], 0xaa);
        let superblock = BiosParameterBlockPacked::from_bytes(&buf).unwrap();
        Fat32 {
            block,
            bpb: superblock,
        }
    }

    pub fn check_fs(&self) {
        info!("Checking FAT32 filesystem");
        info!("{:?}", self.bpb);

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
        assert_eq!(self.bpb.sectors_per_cluster(), 1); // TODO: Make this more generic

        info!("rootdir_base_sec: 0x{:x}", self.rootdir_base_sec());

        info!("Check passed");
    }

    fn rootdir_base_sec(&self) -> u32 {
        self.bpb.reserved_sectors() as u32
            + self.bpb.sectors_per_fat_32() as u32 * self.bpb.fats() as u32
    }

    /// Convert a cluster number to a sector number.
    fn cluster_to_sector(&self, cluster_no: u32) -> u32 {
        self.rootdir_base_sec() + (cluster_no - 2) * self.bpb.sectors_per_cluster() as u32
    }

    pub fn ls_rootdir(&'b mut self) {
        let mut buf = [0; 512];
        self.block
            .read(self.rootdir_base_sec() as usize, &mut buf)
            .unwrap();

        //info!("Rootdir: {:?}", rootdir);

        //let clus_data = self.read_cluster(rootdir.cluster);
        //info!("Cluster data: {:X?}", clus_data);

        {
            let rootdir = DirEntry::root();

            for entry in rootdir.fat_entries(self) {
                println!("{:?}", entry);
            }
        }

        {
            let rootdir = DirEntry::root();
            for data_cluster in rootdir.data_clusters(self) {
                println!("{:?}", data_cluster);
            }
        }

        //let fat_entry = self.get_fat_entry(rootdir.cluster);
        //info!("{:?}", fat_entry);
    }

    // @TODO: This is not very Rusty.
    fn get_fat_entry(&mut self, cluster_no: u32) -> FatEntry {
        let fat_offset = cluster_no * 4;
        let fat_off_sec = self.bpb.reserved_sectors() as u32 + (fat_offset / 512);
        let fat_entry_off = fat_offset as usize % 512;

        let block = self.read_block(fat_off_sec);
        let fat_entry = block[fat_entry_off] as u32
            | (block[fat_entry_off + 1] as u32) << 8
            | (block[fat_entry_off + 2] as u32) << 16
            | (block[fat_entry_off + 3] as u32) << 24;

        FatEntry {
            cluster: cluster_no,
            entry: fat_entry,
        }
    }

    // @TODO
    // This design goes against the zero-copy objective, because
    // we're always returning an owned buffer.
    pub fn read_block(&mut self, sector_no: u32) -> [u8; 512] {
        let mut buf = [0; 512]; // TODO: MaybeUninit?
        self.block.read(sector_no as usize, &mut buf).unwrap();
        buf
    }

    pub fn read_cluster(&mut self, cluster_no: u32) -> [u8; 512] {
        self.read_block(self.cluster_to_sector(cluster_no))
    }
}

/// Represents a location in a FAT32 filesystem.
/// Identified by a cluster number and the offset within that cluster.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct ClusterLoc {
    cluster: u32,
    offset: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct DirEntry {
    loc: ClusterLoc,
}

//type DataClusterIter<'b, B> = core::iter::Map<FatEntries<'b, 'b, B>>;

impl DirEntry {
    fn new(loc: ClusterLoc) -> Self {
        Self { loc }
    }

    /// Returns the "directory entry" of the root directory.
    /// This is a lie, because root directory doesn't have a real directory entry.
    /// So we set cluster to the invalid 0, representing the root directory.
    fn root() -> Self {
        Self::new(ClusterLoc {
            cluster: 0,
            offset: 0,
        })
    }

    fn is_rootdir(&self) -> bool {
        self.loc.cluster == 0
    }

    fn fat_entries<'f, 'b, B: BlockDevice>(
        &self,
        fs: &'f mut Fat32<'b, B>,
    ) -> impl Iterator<Item = FatEntry> + 'f + 'b
    where
        'f: 'b,
    {
        FatEntries {
            fat: fs,
            curr_clus: self.first_data_clus().cluster,
        }
    }

    fn data_clusters<'b, 'f, B: BlockDevice>(
        &self,
        fs: &'f mut Fat32<'b, B>,
    ) -> impl Iterator<Item = u32> + 'f + 'b
    where
        'f: 'b,
    {
        self.fat_entries(fs).map(|e: FatEntry| e.cluster)
    }

    fn first_data_clus(&self) -> ClusterLoc {
        if self.is_rootdir() {
            ClusterLoc {
                cluster: 2,
                offset: 0,
            }
        } else {
            todo!();
        }
    }
}

/// Represents a FAT32 entry.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
struct FatEntry {
    cluster: u32,
    entry: u32,
}

impl FatEntry {
    fn is_free(&self) -> bool {
        self.entry == 0
    }

    /// Is this entry the last entry in a cluster chain?
    fn is_eoc(&self) -> bool {
        self.entry >= 0x0ffffff8 && self.entry <= 0x0fffffff
    }

    fn is_reserved(&self) -> bool {
        self.entry == 0x00000001
    }

    fn is_bad(&self) -> bool {
        self.entry == 0x0ffffff7
    }

    fn has_next(&self) -> bool {
        !self.is_eoc() && !self.is_bad() && !self.is_reserved() && !self.is_free()
    }
}

impl From<FatEntry> for u32 {
    fn from(value: FatEntry) -> Self {
        value.entry
    }
}

/// An iterator over the FAT entries in a FAT32 filesystem.
struct FatEntries<'f, 'b, B>
where
    B: BlockDevice,
{
    fat: &'f mut Fat32<'b, B>,
    curr_clus: u32,
}

// FIXME: What happnes if a dir entry doesn't have a fat entry?
// Meaning it has no data block allocated.
// We could assign meanings to cluster numbers, where 0 means "no data block"?
impl<B> core::iter::Iterator for FatEntries<'_, '_, B>
where
    B: BlockDevice,
{
    type Item = FatEntry;

    fn next(&mut self) -> Option<Self::Item> {
        let curr_clus = self.curr_clus;
        if curr_clus == 0 {
            return None;
        }

        if (FatEntry {
            cluster: 0,
            entry: curr_clus,
        })
        .is_eoc()
        {
            None
        } else {
            let next_clus = self.fat.get_fat_entry(curr_clus);
            self.curr_clus = next_clus.into();
            Some(next_clus)
        }
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
        let bpb: BiosParameterBlockPacked =
            unsafe { core::ptr::read(bytes.as_ptr() as *const Self) };
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
