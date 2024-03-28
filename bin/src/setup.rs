use clap::Parser;
use zkevm::{circuit::DEGREE, utils::load_or_create_params};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specify directory which params have stored in. (default: ./kzg_params)
    #[clap(default_value = "./kzg_params", short, long)]
    params_dir: String,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Args::parse();
    load_or_create_params(&path, *DEGREE).expect("failed to load or create params");
}
