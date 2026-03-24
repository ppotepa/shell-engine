use crate::difficulty::MachineSpec;

#[derive(Debug, Clone)]
pub struct MountEntry {
    pub device: String,
    pub mount_point: String,
    pub fs_type: String,
    pub options: String,
    pub total_kb: u32,
    pub used_kb: u32,
}

pub struct MountTable {
    mounts: Vec<MountEntry>,
}

impl MountTable {
    pub fn new(spec: &MachineSpec) -> Self {
        let root_kb = spec.disk_kb / 2;
        let usr_kb = spec.disk_kb - root_kb;
        let root_used = (spec.disk_kb - spec.disk_free_kb) / 2;
        let usr_used = spec.disk_kb - spec.disk_free_kb - root_used;

        Self {
            mounts: vec![
                MountEntry {
                    device: "/dev/hd1".to_string(),
                    mount_point: "/".to_string(),
                    fs_type: "minix".to_string(),
                    options: "rw".to_string(),
                    total_kb: root_kb,
                    used_kb: root_used,
                },
                MountEntry {
                    device: "/dev/hd2".to_string(),
                    mount_point: "/usr".to_string(),
                    fs_type: "minix".to_string(),
                    options: "rw".to_string(),
                    total_kb: usr_kb,
                    used_kb: usr_used,
                },
            ],
        }
    }

    pub fn get_mounts(&self) -> &[MountEntry] {
        &self.mounts
    }

    pub fn get_mount(&self, point: &str) -> Option<&MountEntry> {
        self.mounts.iter().find(|m| m.mount_point == point)
    }
}
