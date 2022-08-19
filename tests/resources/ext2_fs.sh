root_dir=ext2_fs
img_file=ext2_fs.img

rm -f "$img_file"
/opt/homebrew/opt/e2fsprogs/sbin/mke2fs \
  -L '' \
  -N 0 \
  -O ^64bit \
  -d "$root_dir" \
  -m 5 \
  -r 1 \
  -t ext2 \
  "$img_file" \
  1M \
;
