pub mod deserializer;
pub mod msg;

use rosbag::{ChunkRecord, ChunkRecordsIterator, IndexRecord, RosBag};
use std::collections::HashMap;

pub struct Ros1Bag {
    bag: RosBag,
    conn_id_to_topic: HashMap<u32, String>,
    pub topic_to_type: HashMap<String, String>,
}

impl Ros1Bag {
    pub fn new(bag_path: &str) -> Ros1Bag {
        let bag = RosBag::new(bag_path).expect(".bag file");
        let mut conn_id_to_topic = HashMap::<u32, String>::new();
        let mut topic_to_type = HashMap::<String, String>::new();

        // Iterate over records in the index section
        for record in bag.index_records() {
            if let IndexRecord::Connection(conn) = record.expect("record") {
                topic_to_type.insert(conn.topic.to_string(), conn.tp.to_string());
                conn_id_to_topic.insert(conn.id, conn.topic.to_string());
            }
        }
        Ros1Bag {
            bag,
            conn_id_to_topic,
            topic_to_type,
        }
    }
    pub fn read_messages(&self, topics: &[String]) -> MessageIterator {
        MessageIterator::new(
            self.bag.chunk_records(),
            self.conn_id_to_topic.to_owned(),
            self.topic_to_type.to_owned(),
            topics,
        )
    }
}

pub struct MessageDataOwned {
    pub conn_id: u32,
    pub time: u64,
    pub data: Vec<u8>,
}

pub struct MessageIterator<'a> {
    topics: std::collections::HashSet<String>,
    conn_id_to_topic: HashMap<u32, String>,
    topic_to_type: HashMap<String, String>,
    chunck_iter: ChunkRecordsIterator<'a>,
    current_messages: Option<Vec<MessageDataOwned>>,
}

impl MessageIterator<'_> {
    pub fn new<'a>(
        chunck_iter: ChunkRecordsIterator<'a>,
        conn_id_to_topic: HashMap<u32, String>,
        topic_to_type: HashMap<String, String>,
        topics: &[String],
    ) -> MessageIterator<'a> {
        MessageIterator {
            topics: topics.iter().cloned().collect(),
            chunck_iter,
            conn_id_to_topic,
            topic_to_type,
            current_messages: None,
        }
    }
}

impl<'a> Iterator for MessageIterator<'a> {
    type Item = msg::Msg;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(messages) = &mut self.current_messages {
            if let Some(msg_data) = messages.pop() {
                if let Some(topic) = self.conn_id_to_topic.get(&msg_data.conn_id) {
                    if !self.topics.is_empty() && !self.topics.contains(topic) {
                        return self.next();
                    }
                    let topic_type = self.topic_to_type.get(topic).unwrap().as_str();
                    match topic_type {
                        "sensor_msgs/PointCloud2" => {
                            let pointcloud2: msg::PointCloud2 =
                                deserializer::from_slice(&msg_data.data).expect("");
                            return Some(msg::Msg::PointCloud2(pointcloud2));
                        }
                        _ => {
                            return Some(msg::Msg::Unknown);
                        }
                    }
                } else {
                    return Some(msg::Msg::Unknown);
                }
            } else {
                self.current_messages = None;
                return self.next();
            }
        }
        match self.chunck_iter.next() {
            Some(Ok(ChunkRecord::Chunk(chunk))) => {
                let mut msg_data_vec: Vec<_> = chunk
                    .messages()
                    .filter_map(|x| match x {
                        Ok(rosbag::MessageRecord::MessageData(msg_data)) => {
                            Some(MessageDataOwned {
                                conn_id: msg_data.conn_id,
                                time: msg_data.time,
                                data: msg_data.data.into(),
                            })
                        }
                        _ => None,
                    })
                    .collect();
                msg_data_vec.reverse();
                self.current_messages = Some(msg_data_vec);
                self.next()
            }
            None => None,
            _ => self.next(),
        }
    }
}
