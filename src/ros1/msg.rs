use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    pub sequence_id: u32,
    pub sec: u32,
    pub nsec: u32,
    pub frame_id: String,
}

// MSG: sensor_msgs/PointField
// # This message holds the description of one point entry in the
// # PointCloud2 message format.
// uint8 INT8    = 1
// uint8 UINT8   = 2
// uint8 INT16   = 3
// uint8 UINT16  = 4
// uint8 INT32   = 5
// uint8 UINT32  = 6
// uint8 FLOAT32 = 7
// uint8 FLOAT64 = 8
#[derive(Debug, Serialize, Deserialize)]
pub struct PointField {
    pub name: String,
    pub offset: u32,
    pub datatype: u8,
    pub count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointCloud2 {
    pub header: Header,
    pub height: u32,
    pub width: u32,
    pub fields: Vec<PointField>,

    pub is_bigendian: u8,
    pub point_step: u32,
    pub row_step: u32,
    pub data: Vec<u8>,
    pub is_dense: u8,
}

pub enum Msg {
    PointCloud2(PointCloud2),
    Unknown,
}
