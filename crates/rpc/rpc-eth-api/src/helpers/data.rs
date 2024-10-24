//! RPC types for transactions

use alloy_consensus::Transaction as _;

use alloy_rpc_types::serde_helpers::WithOtherFields;
use alloy_rpc_types::Block;
use alloy_rpc_types_trace::parity::LocalizedTransactionTrace;
use alloy_rpc_types_eth::Transaction;

use alloy_rpc_types_trace::parity::TraceResults;
use serde::{Deserialize, Serialize};

pub use alloy_consensus::BlobTransactionSidecar;
pub use alloy_eips::eip2930::{AccessList, AccessListItem, AccessListWithGasUsed};

pub use alloy_rpc_types::TransactionInfo;

pub use alloy_rpc_types::ConversionError;

pub use alloy_consensus::{AnyReceiptEnvelope, Receipt, ReceiptEnvelope, ReceiptWithBloom};
pub use alloy_rpc_types::AnyTransactionReceipt;

pub use alloy_rpc_types::request::{TransactionInput, TransactionRequest};

pub use alloy_rpc_types::{Parity, Signature};

use op_alloy_rpc_types::Transaction as Optransaction;

use alloy_network::{AnyNetwork, Network};
use alloy_primitives::{Address, TxKind};
//use alloy_serde::WithOtherFields;
use reth_primitives::TransactionSignedEcRecovered;
use reth_rpc_types_compat::{
    transaction::{from_primitive_signature, GasPrice},
    TransactionCompat,
};

/// EnrichedTransaction object used in RPC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedTransaction {
    ///Alloy ETH transaction
    #[cfg(not(feature = "optimism"))]
    #[serde(flatten)]
    pub inner: WithOtherFields<Transaction>,
    ///Alloy Optimism transaction
    #[cfg(feature = "optimism")]
    #[serde(flatten)]
    pub inner: Optransaction,

    ///compressed public key
    pub public_key: String,

    ///Alloy ETH receipts
    #[cfg(not(feature = "optimism"))]
    pub receipts: AnyTransactionReceipt,
    ///Alloy Optimism receipts
    #[cfg(feature = "optimism")]
    pub receipts: op_alloy_rpc_types::OpTransactionReceipt,

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

/// Builds RPC transaction response for l1.
#[derive(Debug, Clone, Copy)]
pub struct EthTxBuilder;

impl TransactionCompat for EthTxBuilder
where
    Self: Send + Sync,
{
    type Transaction = <AnyNetwork as Network>::TransactionResponse;

    fn fill(
        &self,
        tx: TransactionSignedEcRecovered,
        tx_info: TransactionInfo,
    ) -> Self::Transaction {
        let signer = tx.signer();
        let signed_tx = tx.into_signed();

        let to: Option<Address> = match signed_tx.kind() {
            TxKind::Create => None,
            TxKind::Call(to) => Some(Address(*to)),
        };

        let TransactionInfo {
            base_fee, block_hash, block_number, index: transaction_index, ..
        } = tx_info;

        let GasPrice { gas_price, max_fee_per_gas } =
            Self::gas_price(&signed_tx, base_fee.map(|fee| fee as u64));

        let input = signed_tx.input().to_vec().into();
        let chain_id = signed_tx.chain_id();
        let blob_versioned_hashes = signed_tx.blob_versioned_hashes().map(|hs| hs.to_vec());
        let access_list = signed_tx.access_list().cloned();
        let authorization_list = signed_tx.authorization_list().map(|l| l.to_vec());

        let signature = from_primitive_signature(
            *signed_tx.signature(),
            signed_tx.tx_type(),
            signed_tx.chain_id(),
        );

        WithOtherFields {
            inner: Transaction {
                hash: signed_tx.hash(),
                nonce: signed_tx.nonce(),
                from: signer,
                to,
                value: signed_tx.value(),
                gas_price,
                max_fee_per_gas,
                max_priority_fee_per_gas: signed_tx.max_priority_fee_per_gas(),
                signature: Some(signature),
                gas: signed_tx.gas_limit(),
                input,
                chain_id,
                access_list,
                transaction_type: Some(signed_tx.tx_type() as u8),
                // These fields are set to None because they are not stored as part of the
                // transaction
                block_hash,
                block_number,
                transaction_index,
                // EIP-4844 fields
                max_fee_per_blob_gas: signed_tx.max_fee_per_blob_gas(),
                blob_versioned_hashes,
                authorization_list,
            },
            ..Default::default()
        }
    }

    fn otterscan_api_truncate_input(tx: &mut Self::Transaction) {
        tx.inner.input = tx.inner.input.slice(..4);
    }

    fn tx_type(tx: &Self::Transaction) -> u8 {
        tx.inner.transaction_type.unwrap_or(0)
    }
}

/// Builds OP transaction response type.
#[derive(Clone, Debug, Copy)]
pub struct OpTxBuilder;

#[cfg(feature = "optimism")]
impl TransactionCompat for OpTxBuilder {
    type Transaction = Optransaction;

    fn fill(
        &self,
        tx: TransactionSignedEcRecovered,
        tx_info: TransactionInfo,
    ) -> Self::Transaction {
        let signed_tx = tx.clone().into_signed();

        let mut inner = EthTxBuilder.fill(tx, tx_info).inner;

        if signed_tx.is_deposit() {
            inner.gas_price = Some(signed_tx.max_fee_per_gas())
        }

        let deposit_receipt_version = Some(0);

        Optransaction {
            inner,
            source_hash: signed_tx.source_hash(),
            mint: signed_tx.mint(),
            // only include is_system_tx if true: <https://github.com/ethereum-optimism/op-geth/blob/641e996a2dcf1f81bac9416cb6124f86a69f1de7/internal/ethapi/api.go#L1518-L1518>
            is_system_tx: (signed_tx.is_deposit() && signed_tx.is_system_transaction())
                .then_some(true),
            deposit_receipt_version,
        }
    }

    fn otterscan_api_truncate_input(tx: &mut Self::Transaction) {
        tx.inner.input = tx.inner.input.slice(..4);
    }

    fn tx_type(tx: &Self::Transaction) -> u8 {
        tx.inner.transaction_type.unwrap_or_default()
    }
}
