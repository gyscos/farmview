use std::collections::HashMap;

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Data {
    pub hosts: Vec<HostData>,

    pub update_time: String,
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
pub struct Attribute {
    pub value: String,
    pub raw: String,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct DiskData {
    // In bytes
    pub size: Option<usize>,
    // In bytes
    pub available: Option<usize>,
    // In bytes
    pub used: Option<usize>,

    pub mountpoint: String,
    pub name: String,

    pub model: Option<String>,

    pub attrs: Option<HashMap<String, Attribute>>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct NetworkData {
    // In B/s
    #[serde(skip_serializing_if="Option::is_none")]
    pub rx: Option<usize>,

    // In B/s
    #[serde(skip_serializing_if="Option::is_none")]
    pub tx: Option<usize>,

    #[serde(skip_serializing_if="Option::is_none")]
    pub ip: Option<String>,
}
