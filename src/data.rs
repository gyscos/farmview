#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub hosts: Vec<HostData>,

    pub update_time: i64,
}

/// This is what's produced by `fetch_data` regularly.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct HostData {
    #[serde(skip_serializing_if="Option::is_none")]
    pub hostname: Option<String>,

    #[serde(skip_serializing_if="Option::is_none")]
    pub location: Option<String>,

    #[serde(skip_serializing_if="Option::is_none")]
    pub nproc: Option<u8>,
    // Directly from the `uptime` command

    #[serde(skip_serializing_if="Option::is_none")]
    pub uptime: Option<[f32; 3]>,

    #[serde(skip_serializing_if="Option::is_none")]
    pub memory: Option<MemoryData>,

    pub disks: Vec<DiskData>,

    #[serde(skip_serializing_if="Option::is_none")]
    pub power: Option<PowerData>,

    #[serde(skip_serializing_if="Option::is_none")]
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
pub struct PowerData {
    pub current: f32,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DiskData {
    // In bytes
    pub size: usize,
    // In bytes
    pub available: usize,
    // In bytes
    pub used: usize,

    pub mount: String,
    pub device: String,
    pub model: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct NetworkData {
    // In kB/s
    #[serde(skip_serializing_if="Option::is_none")]
    pub rx: Option<f32>,
    // In kB/s
    #[serde(skip_serializing_if="Option::is_none")]
    pub tx: Option<f32>,

    #[serde(skip_serializing_if="Option::is_none")]
    pub ip: Option<String>,
}
