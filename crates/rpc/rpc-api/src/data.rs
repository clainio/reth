//! RPC types for transactions

use alloy_rpc_types;
use alloy_rpc_types_trace::parity::LocalizedTransactionTrace;

use crate::data::alloy_rpc_types::Block;
pub use alloy_rpc_types::other::OtherFields;

use alloy_rpc_types_trace::parity::TraceResults;
use reth_rpc_types::Transaction;
use serde::{Deserialize, Serialize};

pub use alloy_consensus::BlobTransactionSidecar;
pub use alloy_eips::eip2930::{AccessList, AccessListItem, AccessListWithGasUsed};

pub use alloy_rpc_types::TransactionInfo;

pub use alloy_rpc_types::ConversionError;

pub use alloy_rpc_types::OptimismTransactionReceiptFields;

pub use alloy_consensus::{AnyReceiptEnvelope, Receipt, ReceiptEnvelope, ReceiptWithBloom};
pub use alloy_rpc_types::AnyTransactionReceipt;

pub use alloy_rpc_types::request::{TransactionInput, TransactionRequest};

pub use alloy_rpc_types::{Parity, Signature};

/// EnrichedTransaction object used in RPC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedTransaction {
    ///Alloy transaction
    #[serde(flatten)]
    pub inner: Transaction,

    ///compressed public key
    pub public_key: String,

    ///Alloy receipts
    pub receipts: AnyTransactionReceipt,

    ///Alloy traces
    pub trace: TraceResults
}

/// EnrichedBlock object used in RPC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedBlock{
    ///Alloy block
    #[serde(flatten)]
    pub inner: Block<EnrichedTransaction>,

    ///static block rewards
    pub rewards: Vec<LocalizedTransactionTrace>
}