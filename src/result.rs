struct Data {
    hosts: Vec<HostData>,
    updateTime: String,
}

struct HostData {
    pub name: String,
    pub online: bool,
    pub responsive: bool,
    pub ping: String,
    pub ncpu: u8,
    pub load: [f64; 3],
    pub cpu_usage: u8,
    pub ram_usage: MemoryData,
    pub disk_usage: Vec<DiskData>,
}

pub struct MemoryData {
    pub total_h: String,
    pub total_k: u64,
    pub used_k: u64,
    pub percent_used: u8,
}

pub struct DiskData {
    pub device: String,
    pub mount: String,
    pub total_h: String,
    pub total_k: u64,
    pub used_k: u64,
    percent_used: u8,
}
