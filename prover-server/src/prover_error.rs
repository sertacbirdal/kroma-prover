use jsonrpc_http_server::jsonrpc_core::{Error as JsonError, ErrorCode as JsonErrorCode};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use std::env;
use zkevm::circuit::MAX_TXS;

use crate::utils::kroma_err;

/// Error Code
#[derive(Debug)]
pub enum ErrorCode {
    /// Cannot find the path for KZG parameters.
    KZGParamsNotFound,
    /// Received a trace that is invalid JSON.
    TraceParseError,
    /// Received a Trace with a different chain id from the server.
    ChainIdNotMatched,
    /// Received a trace containing transactions that exceed `MAX_TXS`.
    TooManyTxs,
}

impl ErrorCode {
    pub fn code(&self) -> i64 {
        match *self {
            // Human error starts with `1`
            ErrorCode::KZGParamsNotFound => 1000,
            // Trace error starts with `2`
            ErrorCode::TraceParseError => 2000,
            ErrorCode::ChainIdNotMatched => 2001,
            // Spec. error starts with `3`
            ErrorCode::TooManyTxs => 3000,
        }
    }
}

impl From<i64> for ErrorCode {
    fn from(code: i64) -> Self {
        match code {
            1000 => ErrorCode::KZGParamsNotFound,
            2000 => ErrorCode::TraceParseError,
            2001 => ErrorCode::ChainIdNotMatched,
            3000 => ErrorCode::TooManyTxs,
            _ => panic!("not supported code: {:?}", code),
        }
    }
}

impl<'a> Deserialize<'a> for ErrorCode {
    fn deserialize<D>(deserializer: D) -> Result<ErrorCode, D::Error>
    where
        D: Deserializer<'a>,
    {
        let code: i64 = Deserialize::deserialize(deserializer)?;
        Ok(ErrorCode::from(code))
    }
}

impl Serialize for ErrorCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_i64(self.code())
    }
}

/// Error object as defined in Spec
#[derive(Debug)]
pub struct ProverError {
    /// Code
    pub code: ErrorCode,
    /// Message
    pub message: Option<String>,
}

impl ProverError {
    /// Wraps given `ErrorCode`
    pub fn new(code: ErrorCode, message: Option<String>) -> Self {
        ProverError { code, message }
    }

    /// Creates new `KZGParamsNotFound`
    pub fn kzg_params_not_found() -> Self {
        let err = Self::new(ErrorCode::KZGParamsNotFound, None);
        kroma_err(err.to_string());
        err
    }

    /// Creates new `TraceParseError`
    pub fn trace_parse_error(msg: String) -> Self {
        let err = Self::new(ErrorCode::TraceParseError, Some(msg));
        kroma_err(err.to_string());
        err
    }

    /// Creates new `ChainIdNotMatched`
    pub fn chain_id_not_matched(trace_chain_id: u64) -> Self {
        let server_chain_id = env::var("CHAIN_ID").unwrap().parse::<u32>().unwrap();
        let msg = format!(
            "ChainId not matched, expected({:?}), actual({:?})",
            server_chain_id, trace_chain_id
        );
        let err = Self::new(ErrorCode::ChainIdNotMatched, Some(msg));
        kroma_err(err.to_string());
        err
    }

    /// Creates new `TooManyTxs`
    pub fn too_many_txs(trace_tx_num: usize) -> Self {
        let msg = format!(
            "Too may txs, max_txs({:?}), actual({:?})",
            MAX_TXS, trace_tx_num
        );
        let err = Self::new(ErrorCode::TooManyTxs, Some(msg));
        kroma_err(err.to_string());
        err
    }
}

impl std::fmt::Display for ProverError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for ProverError {}

impl From<ProverError> for JsonError {
    fn from(err: ProverError) -> Self {
        Self {
            code: JsonErrorCode::InternalError,
            message: err.to_string(),
            data: None,
        }
    }
}
