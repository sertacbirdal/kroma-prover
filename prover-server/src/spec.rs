use serde_derive::{Deserialize, Serialize};
use zkevm::circuit::{AGG_DEGREE, CHAIN_ID, DEGREE, MAX_CALLDATA, MAX_TXS};

#[derive(Debug, Serialize, Deserialize)]
pub struct ZkSpec {
    pub degree: u32,
    pub agg_degree: u32,
    pub chain_id: u32,
    pub max_txs: u32,
    pub max_call_data: u32,
}

impl Default for ZkSpec {
    fn default() -> Self {
        Self {
            degree: *DEGREE as u32,
            agg_degree: *AGG_DEGREE as u32,
            chain_id: *CHAIN_ID as u32,
            max_txs: MAX_TXS as u32,
            max_call_data: MAX_CALLDATA as u32,
        }
    }
}

impl ZkSpec {
    pub fn new(chain_id: u32) -> Self {
        Self {
            degree: *DEGREE as u32,
            agg_degree: *AGG_DEGREE as u32,
            chain_id,
            max_txs: MAX_TXS as u32,
            max_call_data: MAX_CALLDATA as u32,
        }
    }
}
