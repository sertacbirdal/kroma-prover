use clap::Parser;
use zkevm::{
    circuit::{AGG_DEGREE, DEGREE},
    utils::create_kzg_params_to_file,
};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specify directory which params have stored in. (default: ./kzg_params)
    #[clap(default_value = "./kzg_params", short, long)]
    params_dir: String,
    /// Specify domain size. (generate 2 params with `DEGREE`/`AGG_DEGREE` if it is omitted.)
    #[clap(default_value_t = 0, short)]
    n: usize,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let args = Args::parse();
    let params_dir = &args.params_dir;
    if args.n == 0 {
        let _ = create_kzg_params_to_file(params_dir, *DEGREE);
        let _ = create_kzg_params_to_file(params_dir, *AGG_DEGREE);
    } else if args.n > 30 {
        panic!("too big domain size, you should enter `n` less than 30");
    } else {
        let _ = create_kzg_params_to_file(params_dir, args.n);
    }
}
