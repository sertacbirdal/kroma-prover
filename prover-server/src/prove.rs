use crate::prover_error::ProverError;
use crate::utils::{kroma_info, kroma_msg};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use types::eth::BlockTrace;
use utils::Measurer;
use zkevm::circuit::{AGG_DEGREE, DEGREE};
use zkevm::prover::{AggCircuitProof, Prover};
use zkevm::utils::{load_kzg_params, load_or_create_seed};

const PARAMS_DIR: &str = "./kzg_params/";
const SEED_FILE: &str = "./rng_seed";
const OUT_PROOF_DIR: &str = "./out_proof/";

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProofResult {
    pub final_pair: Option<Vec<u8>>,
    pub proof: Vec<u8>,
}

impl ProofResult {
    pub fn new(proof: Vec<u8>, final_pair: Option<Vec<u8>>) -> Self {
        Self { proof, final_pair }
    }
}

pub fn create_proof(trace: BlockTrace) -> Result<ProofResult, ProverError> {
    // load or create material for prover
    let params = load_kzg_params(PARAMS_DIR, *DEGREE);
    let agg_params = load_kzg_params(PARAMS_DIR, *AGG_DEGREE);
    if params.is_err() || agg_params.is_err() {
        return Err(ProverError::kzg_params_not_found());
    }
    let params = params.unwrap();
    let agg_params = agg_params.unwrap();

    let seed = load_or_create_seed(SEED_FILE)
        .unwrap_or_else(|_| panic!("{}", kroma_msg("failed to load or create seed")));

    // prepare directory to store proof. (i.e., ./out_proof/<block_number>/)
    let height_hex = trace.header.number.unwrap().to_string();
    let out_dir = PathBuf::from(OUT_PROOF_DIR).join(height_hex);
    let _ = create_dir_all(&out_dir);

    // build prover
    let mut prover = Prover::from_params_and_seed(params, Some(agg_params), seed);
    // specify the dir to store the vk and proof of the intermediate circuit.
    prover.debug_dir = out_dir.to_str().unwrap().to_string();

    create_agg_proof(prover, trace)
}

pub fn create_agg_proof(mut prover: Prover, trace: BlockTrace) -> Result<ProofResult, ProverError> {
    kroma_info("start creating proof");

    // generate proof
    let mut timer = Measurer::new();
    let proof = prover
        .create_agg_circuit_proof(&trace)
        .unwrap_or_else(|_| panic!("{}", kroma_msg("cannot generate agg_proof")));
    timer.end(&kroma_msg("finish generating a proof"));

    // store proof and verifier contract as files
    let dir = PathBuf::from(prover.debug_dir.clone());
    write_agg_proof(&dir, &proof);
    kroma_info(format!("output files to {}", dir.to_str().unwrap()));

    let proof_result = ProofResult::new(proof.proof.clone(), Some(proof.final_pair));
    Ok(proof_result)
}

pub fn write_agg_proof(dir: &Path, proof: &AggCircuitProof) {
    let mut proof_path = dir.join("agg.proof");
    let _ = fs::create_dir_all(&proof_path);

    proof.write_to_dir(&mut proof_path);
}
