use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    sequence_id: u32,
    sec: u32,
    nsec: u32,
    frame_id: String,
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
    offset: u32,
    datatype: u8,
    count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PointCloud2 {
    pub header: Header,
    pub height: u32,
    pub width: u32,
    pub fields: Vec<PointField>,

    is_bigendian: u8,
    point_step: u32,
    row_step: u32,
    data: Vec<u8>,
    is_dense: u8,
}

pub enum Msg {
    PointCloud2(PointCloud2),
    Unknown,
}
