//! RPC types for transactions

use alloy_consensus::Signed;
use alloy_consensus::Transaction as _;

use alloy_consensus::TxEip4844Variant;
use alloy_consensus::TxEnvelope;
use alloy_network::{Ethereum, Network};

use alloy_rpc_types::Block;
use alloy_rpc_types_trace::parity::LocalizedTransactionTrace;
use alloy_rpc_types_eth::Transaction;
use reth_primitives::{TransactionSigned, TransactionSignedEcRecovered};

use alloy_rpc_types_trace::parity::TraceResults;
use serde::{Deserialize, Serialize};

pub use alloy_consensus::BlobTransactionSidecar;
pub use alloy_eips::eip2930::{AccessList, AccessListItem, AccessListWithGasUsed};

pub use alloy_rpc_types::TransactionInfo;

pub use alloy_rpc_types::ConversionError;

pub use alloy_consensus::{AnyReceiptEnvelope, Receipt, ReceiptEnvelope, ReceiptWithBloom};
pub use alloy_rpc_types::AnyTransactionReceipt;

pub use alloy_rpc_types::request::{TransactionInput, TransactionRequest};

#[cfg(feature = "optimism")]
use op_alloy_consensus::OpTxEnvelope;
#[cfg(feature = "optimism")]
use op_alloy_rpc_types::Transaction as Optransaction;

use reth_rpc_types_compat::TransactionCompat;

/// EnrichedTransaction object used in RPC
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnrichedTransaction {
    ///Alloy ETH transaction
    #[cfg(not(feature = "optimism"))]
    #[serde(flatten)]
    pub inner: Transaction,
    ///Alloy Optimism transaction
    #[cfg(feature = "optimism")]
    #[serde(flatten)]
    pub inner: Optransaction,

    ///compressed public key
    pub public_key: String,

    ///Alloy ETH receipts
    #[cfg(not(feature = "optimism"))]
    pub receipts: alloy_rpc_types_eth::TransactionReceipt,
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
    type Transaction = <Ethereum as Network>::TransactionResponse;

    fn fill(
        &self,
        tx: TransactionSignedEcRecovered,
        tx_info: TransactionInfo,
    ) -> Self::Transaction {
        let from = tx.signer();
        let TransactionSigned { transaction, signature, hash } = tx.into_signed();

        let inner: TxEnvelope = match transaction {
            reth_primitives::Transaction::Legacy(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Eip2930(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Eip1559(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Eip4844(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Eip7702(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            #[allow(unreachable_patterns)]
            _ => unreachable!(),
        };

        let TransactionInfo {
            block_hash, block_number, index: transaction_index, base_fee, ..
        } = tx_info;

        let effective_gas_price = base_fee
            .map(|base_fee| {
                inner.effective_tip_per_gas(base_fee as u64).unwrap_or_default() + base_fee
            })
            .unwrap_or_else(|| inner.max_fee_per_gas());

        Transaction {
            inner,
            block_hash,
            block_number,
            transaction_index,
            from,
            effective_gas_price: Some(effective_gas_price),
        }
    }

    fn otterscan_api_truncate_input(tx: &mut Self::Transaction) {
        let input = match &mut tx.inner {
            TxEnvelope::Eip1559(tx) => &mut tx.tx_mut().input,
            TxEnvelope::Eip2930(tx) => &mut tx.tx_mut().input,
            TxEnvelope::Legacy(tx) => &mut tx.tx_mut().input,
            TxEnvelope::Eip4844(tx) => match tx.tx_mut() {
                TxEip4844Variant::TxEip4844(tx) => &mut tx.input,
                TxEip4844Variant::TxEip4844WithSidecar(tx) => &mut tx.tx.input,
            },
            TxEnvelope::Eip7702(tx) => &mut tx.tx_mut().input,
            _ => return,
        };
        *input = input.slice(..4);
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
        let from = tx.signer();
        let TransactionSigned { transaction, signature, hash } = tx.into_signed();

        let inner = match transaction {
            reth_primitives::Transaction::Legacy(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Eip2930(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Eip1559(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Eip4844(_) => unreachable!(),
            reth_primitives::Transaction::Eip7702(tx) => {
                Signed::new_unchecked(tx, signature, hash).into()
            }
            reth_primitives::Transaction::Deposit(tx) => OpTxEnvelope::Deposit(tx),
        };

        let deposit_receipt_version = Some(0);

        let TransactionInfo {
            block_hash, block_number, index: transaction_index, base_fee, ..
        } = tx_info;

        let effective_gas_price = base_fee
            .map(|base_fee| {
                inner.effective_tip_per_gas(base_fee as u64).unwrap_or_default() + base_fee
            })
            .unwrap_or_else(|| inner.max_fee_per_gas());

        Optransaction {
            inner: alloy_rpc_types::Transaction {
                inner,
                block_hash,
                block_number,
                transaction_index,
                from,
                effective_gas_price: Some(effective_gas_price),
            },
            deposit_receipt_version,
        }
    }

    fn otterscan_api_truncate_input(tx: &mut Self::Transaction) {
        let input = match &mut tx.inner.inner {
            OpTxEnvelope::Eip1559(tx) => &mut tx.tx_mut().input,
            OpTxEnvelope::Eip2930(tx) => &mut tx.tx_mut().input,
            OpTxEnvelope::Legacy(tx) => &mut tx.tx_mut().input,
            OpTxEnvelope::Eip7702(tx) => &mut tx.tx_mut().input,
            OpTxEnvelope::Deposit(tx) => &mut tx.input,
            _ => return,
        };
        *input = input.slice(..4);
    }

}
