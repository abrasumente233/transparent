set -eu

mkdir -p build

hdd=fs.img

echo '[fs: 1/3] creating disk file'
rm -r $hdd
dd if=/dev/zero of=$hdd bs=1M count=48 > /dev/null 2>&1

echo '[fs: 2/3] partitioning & formatting disk'
echo "n
p


t
b
a
w
"|fdisk $hdd > /dev/null 2>&1 ;mkfs.vfat -F 32 $hdd > /dev/null 2>&1

echo '[fs: 3/3] copying files to the disk'
mkdir -p fs
sudo mount $hdd fs/

sudo cp README fs/README.TXT
sudo cp README fs/this_is_a_file_with_really_lOOOOOOOOOOOOOOOOg_name.txt
sudo mkdir fs/TEST_DIR
sudo cp README fs/TEST_DIR/POEM.TXT
echo 'hey, my name is file 1' | sudo tee -a fs/FILE1.TXT > /dev/null
echo 'hey, my name is file 2' | sudo tee -a fs/FILE2.TXT > /dev/null
echo 'hey, my name is file 3' | sudo tee -a fs/FILE3.TXT > /dev/null
echo 'hey, my name is file 4' | sudo tee -a fs/FILE4.TXT > /dev/null
echo 'hey, my name is file 5' | sudo tee -a fs/FILE5.TXT > /dev/null
echo 'hey, my name is file 6' | sudo tee -a fs/FILE6.TXT > /dev/null
echo 'hey, my name is file 7' | sudo tee -a fs/FILE7.TXT > /dev/null
echo 'hey, my name is file 8' | sudo tee -a fs/FILE8.TXT > /dev/null
echo 'hey, my name is file 9' | sudo tee -a fs/FILE9.TXT > /dev/null
echo 'hey, my name is file 10' | sudo tee -a fs/FILE10.TXT > /dev/null
echo 'hey, my name is file 11' | sudo tee -a fs/FILE11.TXT > /dev/null
echo 'hey, my name is file 12' | sudo tee -a fs/FILE12.TXT > /dev/null
echo 'hey, my name is file 13' | sudo tee -a fs/FILE13.TXT > /dev/null
echo 'hey, my name is file 14' | sudo tee -a fs/FILE14.TXT > /dev/null
echo 'hey, my name is file 15' | sudo tee -a fs/FILE15.TXT > /dev/null
echo 'hey, my name is file 16' | sudo tee -a fs/FILE16.TXT > /dev/null
echo 'hey, my name is file 17' | sudo tee -a fs/FILE17.TXT > /dev/null
echo 'hey, my name is file 18' | sudo tee -a fs/FILE18.TXT > /dev/null
echo 'hey, my name is file 19' | sudo tee -a fs/FILE19.TXT > /dev/null
echo 'hey, my name is file 20' | sudo tee -a fs/FILE20.TXT > /dev/null
echo 'hey, my name is file 21' | sudo tee -a fs/FILE21.TXT > /dev/null
echo 'hey, my name is file 22' | sudo tee -a fs/FILE22.TXT > /dev/null
echo 'hey, my name is file 23' | sudo tee -a fs/FILE23.TXT > /dev/null

sudo umount fs/
