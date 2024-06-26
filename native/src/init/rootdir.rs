use std::fs::File;
use std::io::Write;
use std::mem;
use std::os::fd::{FromRawFd, RawFd};

use base::{debug, Utf8CStr};

pub fn inject_magisk_rc(fd: RawFd, tmp_dir: &Utf8CStr) {
    debug!("Injecting magisk rc");

    let mut file = unsafe { File::from_raw_fd(fd) };

    write!(
        file,
        r#"
service su_daemon /dev/xu --daemon
    group root
    seclabel {0}
    user root

on post-fs-data
    start logd
    start su_daemon 
    exec {0} 0 0 -- {1}/magisk --post-fs-data

on init
    exec {0} 0 0 -- /system/bin/sh -c "/system/bin/cat /vendor/etc/fstab.default |/system/bin/sed 's/fileencryption/fillencryption/g' > /dev/fstab && chcon u:object_r:vendor_configs_file:s0 /dev/fstab && /system/bin/chmod 0644 /dev/fstab && /system/bin/mount -o bind /dev/fstab /vendor/etc/fstab.default"

on property:vold.decrypt=trigger_restart_framework
    exec {0} 0 0 -- {1}/magisk --service

on nonencrypted
    exec {0} 0 0 -- {1}/magisk --service

on property:sys.boot_completed=1
    exec {0} 0 0 -- {1}/magisk --boot-complete
    exec {0} 0 0 -- /system/bin/swapoff /dev/block/zram0

on property:init.svc.zygote=stopped
    exec {0} 0 0 -- {1}/magisk --zygote-restart
"#,
        "u:r:magisk:s0", tmp_dir
    )
    .ok();

    mem::forget(file)
}
