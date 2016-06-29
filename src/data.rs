#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub hosts: Vec<Option<HostData>>,
}

/// This is what's produced by `fetch_data` regularly.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct HostData {
    pub hostname: Option<String>,
    pub nproc: Option<u8>,
    // Directly from the `uptime` command
    pub uptime: Option<[f32; 3]>,
    pub memory: Option<MemoryData>,
    pub disks: Vec<DiskData>,
    pub network: Option<NetworkData>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MemoryData {
    // In kiB
    pub total: usize,
    // In kiB
    pub used: usize,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DiskData {
    // In kiB
    pub size: usize,
    // In kiB
    pub available: usize,

    pub mount: String,
    pub device: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct NetworkData {
    // In MB/s
    pub rx: f32,
    // In MB/s
    pub tx: f32,
}
