//! Loads chain metadata.

use std::sync::Arc;

use alloy_rpc_types_trace::parity::LocalizedTransactionTrace;
use futures::Future;
use reth_chainspec::{ChainInfo, ChainSpec};
use reth_errors::RethResult;
use reth_primitives::{Address, BlockId, SealedBlock, U64};
use reth_rpc_eth_types::EthResult;
use reth_rpc_types::{trace::parity::TraceResultsWithTransactionHash, BlockNumberOrTag, SyncStatus};

/// `Eth` API trait.
///
/// Defines core functionality of the `eth` API implementation.
#[auto_impl::auto_impl(&, Arc)]
pub trait EthApiSpec: Send + Sync {
    /// Returns the current ethereum protocol version.
    fn protocol_version(&self) -> impl Future<Output = RethResult<U64>> + Send;

    /// Returns the chain id
    fn chain_id(&self) -> U64;

    /// Returns provider chain info
    fn chain_info(&self) -> RethResult<ChainInfo>;

    /// Returns a list of addresses owned by provider.
    fn accounts(&self) -> Vec<Address>;

    /// Returns `true` if the network is undergoing sync.
    fn is_syncing(&self) -> bool;

    /// Returns the [`SyncStatus`] of the network
    fn sync_status(&self) -> RethResult<SyncStatus>;

    /// Returns the configured [`ChainSpec`].
    fn chain_spec(&self) -> Arc<ChainSpec>;

    /// Replays all transactions in a block
    fn get_trx_trace(&self, block_number: BlockNumberOrTag) -> impl Future< Output = EthResult<Option<Vec<TraceResultsWithTransactionHash>>>> + Send;

    ///Returns SealedBlock by id
    fn get_block_by_id(&self, block_id: BlockId) -> impl Future<Output = EthResult<Option<SealedBlock>>> + Send;

    /// Returns author and uncle rewards at a given block.
    fn get_block_rewards(&self, block:&SealedBlock) -> impl Future<Output = EthResult<Option<Vec<LocalizedTransactionTrace>>>> + Send;
}
