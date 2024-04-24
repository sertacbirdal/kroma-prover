use clap::{ArgEnum, Parser};
use halo2_proofs::{consts::SEED, halo2curves::bn256::Bn256, poly::kzg::commitment::ParamsKZG};
use std::{
    collections::HashMap,
    ffi::OsString,
    fs::{self, File},
    io::Write,
    path::PathBuf,
};
use types::eth::BlockTrace;
use utils::{check_chain_id, is_tachyon, Measurer};
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
    /// Specify whether to create `Verifier.sol`. (default: true)
    #[clap(default_value_t = true, short, long = "gen_sol")]
    gen_sol: bool,
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

    fn load_params(&self) -> (ParamsKZG<Bn256>, Option<ParamsKZG<Bn256>>) {
        let params = load_kzg_params(&self.params_dir, *DEGREE).expect("failed to load kzg params");
        let agg_params = match self.circuit {
            CircuitType::AGG => {
                let params = load_kzg_params(&self.params_dir, *AGG_DEGREE)
                    .expect("failed to load kzg agg params");
                Some(params)
            }
            _ => None,
        };
        (params, agg_params)
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
    let is_tachyon = is_tachyon();
    log::info!("chain_id: {chain_id}, tachyon: {is_tachyon}");
    let args = Args::parse();

    // Prepare KZG params and rng for prover
    let mut timer = Measurer::new();
    let (params, agg_params) = args.load_params();
    let mut prover = Prover::from_params_and_seed(params, agg_params, SEED);
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

                if args.gen_sol {
                    let sol = prover.create_solidity_verifier(&agg_proof);
                    write_file(
                        &mut out_dir,
                        "verifier.sol",
                        &Vec::<u8>::from(sol.as_bytes()),
                    );
                    log::info!("verifier solidity to {}", out_dir.to_str().unwrap());
                }
                log::info!("output files to {}", out_dir.to_str().unwrap());
            }
        }
        timer.end("finish generating a proof");
    }
    outer_timer.end("finish generating all");
}
