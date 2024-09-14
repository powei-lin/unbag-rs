pub mod ros1;

use ros1::msg;

pub fn unbag_ros1(file_path: &str, output_folder: &str) {
    let bag = ros1::Ros1Bag::new(file_path);
    for m in bag.read_messages(&[]) {
        match m {
            msg::Msg::PointCloud2(p) => {
                for f in p.fields {
                    println!("{}", f.name);
                }
            }
            msg::Msg::Unknown => {}
        }
        // println!("a")
    }
}
