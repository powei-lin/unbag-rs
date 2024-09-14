use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct UnbagCli {
    bag_path: String,

    #[arg(short, long, default_value_t = String::from("output"))]
    output: String,
}

fn main() {
    let cli = UnbagCli::parse();
    unbag_rs::unbag_ros1(&cli.bag_path, &cli.output);
}
