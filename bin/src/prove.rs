use clap::{ArgEnum, Parser};
use halo2_proofs::consts::SEED;
use rand::SeedableRng;
use rand_xorshift::XorShiftRng;
use std::collections::HashMap;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use types::eth::BlockTrace;
use utils::check_chain_id;
use utils::Measurer;
use zkevm::{
    circuit::{EvmCircuit, StateCircuit, AGG_DEGREE, DEGREE, MAX_TXS},
    io::write_file,
    prover::Prover,
    utils::{get_block_trace_from_file, load_kzg_params},
};

#[derive(ArgEnum, Debug, Clone, PartialEq)]
#[clap(rename_all = "kebab_case")]
enum CircuitType {
    EVM,
    STATE,
    AGG,
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Specify directory which params have stored in. (default: ./kzg_params)
    #[clap(default_value = "./kzg_params", short, long)]
    params_dir: String,
    /// Specify path to block trace. (json file or directory)
    #[clap(short, long)]
    trace_path: String,
    /// Specify circuit type in [evm, state, agg].
    #[clap(short, long, arg_enum)]
    circuit: CircuitType,
}

impl Args {
    fn load_traces(&self) -> HashMap<OsString, BlockTrace> {
        let mut traces = HashMap::new();
        let trace_path = PathBuf::from(&self.trace_path);
        if trace_path.is_dir() {
            for entry in fs::read_dir(trace_path).unwrap() {
                let path = entry.unwrap().path();
                if path.is_file() && path.to_str().unwrap().ends_with(".json") {
                    let block_trace = get_block_trace_from_file(path.to_str().unwrap());
                    Args::panic_if_tx_too_many(&block_trace);
                    traces.insert(path.file_stem().unwrap().to_os_string(), block_trace);
                }
            }
        } else {
            let block_trace = get_block_trace_from_file(trace_path.to_str().unwrap());
            Args::panic_if_tx_too_many(&block_trace);
            traces.insert(trace_path.file_stem().unwrap().to_os_string(), block_trace);
        }
        traces
    }

    fn panic_if_tx_too_many(trace: &BlockTrace) {
        let tx_count = trace.transactions.len();
        if tx_count > MAX_TXS {
            panic!(
                "{}",
                format!(
                    "too many transactions. MAX_TXS: {}, given transactions: {}",
                    MAX_TXS, tx_count
                )
            );
        }
    }
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let chain_id = check_chain_id();
    log::info!("chain_id: {chain_id}");
    let args = Args::parse();

    // Prepare KZG params and rng for prover
    let mut timer = Measurer::new();
    let params = load_kzg_params(&args.params_dir, *DEGREE).expect("failed to load kzg params");
    let agg_params =
        load_kzg_params(&args.params_dir, *AGG_DEGREE).expect("failed to load kzg agg params");
    let rng = XorShiftRng::from_seed(SEED);

    let mut prover = Prover::from_params_and_rng(params, agg_params, rng);
    timer.end("finish loading params");

    // Getting traces from specific directory
    let traces = args.load_traces();

    // Generating proofs for each trace
    let mut outer_timer = Measurer::new();
    for (trace_name, trace) in traces {
        let mut out_dir = PathBuf::from(&trace_name);
        fs::create_dir_all(&out_dir).unwrap();
        prover.debug_dir = String::from(out_dir.to_str().unwrap());

        timer.start();
        match args.circuit {
            CircuitType::EVM => {
                let proof_path = PathBuf::from(&trace_name).join("evm.proof");
                let evm_proof = prover
                    .create_target_circuit_proof::<EvmCircuit>(&trace)
                    .expect("cannot generate evm_proof");
                let mut f = File::create(&proof_path).unwrap();
                f.write_all(evm_proof.proof.as_slice()).unwrap();
            }
            CircuitType::STATE => {
                let proof_path = PathBuf::from(&trace_name).join("state.proof");
                let state_proof = prover
                    .create_target_circuit_proof::<StateCircuit>(&trace)
                    .expect("cannot generate state_proof");
                let mut f = File::create(&proof_path).unwrap();
                f.write_all(state_proof.proof.as_slice()).unwrap();
            }
            CircuitType::AGG => {
                let mut proof_path = PathBuf::from(&trace_name).join("agg.proof");
                let agg_proof = prover
                    .create_agg_circuit_proof(&trace)
                    .expect("cannot generate agg_proof");
                fs::create_dir_all(&proof_path).unwrap();
                agg_proof.write_to_dir(&mut proof_path);

                let sol = prover.create_solidity_verifier(&agg_proof);
                write_file(
                    &mut out_dir,
                    "verifier.sol",
                    &Vec::<u8>::from(sol.as_bytes()),
                );
                log::info!("output files to {}", out_dir.to_str().unwrap());
            }
        }
        timer.end("finish generating a proof");
    }
    outer_timer.end("finish generating all");
}
