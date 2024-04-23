use clap::Parser;
use jsonrpc_derive::rpc;
use jsonrpc_http_server::jsonrpc_core::{Error as JsonError, Result as JsonResult};
use jsonrpc_http_server::ServerBuilder;
use prove::{create_proof, ProofResult};
use prover_server::prove;
use prover_server::prover_error::ProverError;
use prover_server::spec::ZkSpec;
use prover_server::utils::kroma_info;
use types::eth::BlockTrace;
use utils::check_chain_id;
use zkevm::circuit::{CHAIN_ID, MAX_TXS};

#[rpc]
pub trait Rpc {
    #[rpc(name = "spec")]
    /// return the prover's specification as JSON String.
    ///
    /// # Returns
    ///
    /// String of ZkSpec instance which includes below
    /// 1. pub degree: u32,
    /// 2. pub agg_degree: u32,
    /// 3. pub chain_id: u32,
    /// 4. pub max_txs: u32,
    /// 5. pub max_call_data: u32,
    fn spec(&self) -> JsonResult<ZkSpec> {
        let spec = ZkSpec::new(*CHAIN_ID as u32);
        Ok(spec)
    }

    #[rpc(name = "prove")]
    /// return proof related to the trace.
    fn prove(&self, trace: String) -> JsonResult<ProofResult>;
}

pub struct RpcImpl;

impl Rpc for RpcImpl {
    /// return zk-proof generated with the trace as an input.
    ///
    /// # Arguments
    /// * `trace` - A trace of the specific block as a JSON String.
    ///
    /// # Returns
    /// ProofResult instance which includes proof and final pair.
    fn prove(&self, trace: String) -> JsonResult<ProofResult> {
        // initiate BlockTrace
        let block_trace: BlockTrace = match serde_json::from_slice(trace.as_bytes()) {
            Ok(trace) => trace,
            Err(e) => {
                let err = ProverError::trace_parse_error(e.to_string());
                return JsonResult::Err(JsonError::from(err));
            }
        };

        // check number of txs in the trace
        let tx_count = block_trace.transactions.len();
        if tx_count > MAX_TXS {
            let err = ProverError::too_many_txs(tx_count);
            return JsonResult::Err(JsonError::from(err));
        }

        // check chain id
        let trace_chain_id = block_trace.chain_id.as_u64();
        if *CHAIN_ID != trace_chain_id {
            let err = ProverError::chain_id_not_matched(trace_chain_id);
            return JsonResult::Err(JsonError::from(err));
        }

        match create_proof(block_trace) {
            Ok(result) => JsonResult::Ok(result),
            Err(e) => JsonResult::Err(JsonError::from(e)),
        }
    }
}

pub struct MockRpcImpl;

impl Rpc for MockRpcImpl {
    /// Regardless of the received trace, it returns a zero proof.
    fn prove(&self, _trace: String) -> JsonResult<ProofResult> {
        kroma_info("return zero proof");
        Ok(ProofResult::new(vec![0; 4640], Some(vec![0; 128])))
    }
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long = "endpoint")]
    endpoint: Option<String>,
}

fn main() {
    dotenv::dotenv().ok();
    env_logger::init();

    let chain_id = check_chain_id();
    let args = Args::parse();
    let endpoint = args.endpoint.unwrap_or("127.0.0.1:3030".to_string());

    let mut io = jsonrpc_core::IoHandler::new();
    #[cfg(not(feature = "mock-server"))]
    io.extend_with(RpcImpl.to_delegate());
    #[cfg(feature = "mock-server")]
    io.extend_with(MockRpcImpl.to_delegate());

    kroma_info(format!(
        "Prover server starting on {endpoint}. CHAIN_ID: {chain_id}"
    ));
    let server = ServerBuilder::new(io)
        .threads(3)
        .max_request_body_size(32_000_000)
        .start_http(&endpoint.parse().unwrap())
        .unwrap();

    server.wait();
}
