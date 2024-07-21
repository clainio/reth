use std::sync::Arc;

use reth_chainspec::{ChainInfo, ChainSpec};
use reth_errors::{RethError, RethResult};
use reth_evm::ConfigureEvm;
use reth_network_api::NetworkInfo;
use reth_primitives::{Address, SealedBlock, U256, U64};
use reth_provider::{BlockNumReader, BlockReaderIdExt, ChainSpecProvider, EvmEnvProvider, StateProviderFactory};
use reth_rpc_eth_api::helpers::{EthApiSpec, LoadBlock};
use reth_rpc_eth_types::EthResult;
use reth_rpc_types::{trace::parity::{LocalizedTransactionTrace, TraceResultsWithTransactionHash, TraceType}, BlockId, BlockNumberOrTag, SyncInfo, SyncStatus};
use reth_transaction_pool::TransactionPool;
use revm_primitives::HashSet;

use crate::EthApi;

impl<Provider, Pool, Network, EvmConfig> EthApiSpec for EthApi<Provider, Pool, Network, EvmConfig>
where
    Pool: TransactionPool + 'static,
    Provider:
        BlockReaderIdExt + ChainSpecProvider + StateProviderFactory + EvmEnvProvider + 'static,
    Network: NetworkInfo + 'static,
    EvmConfig: ConfigureEvm,
{
    /// Returns the current ethereum protocol version.
    ///
    /// Note: This returns an [`U64`], since this should return as hex string.
    async fn protocol_version(&self) -> RethResult<U64> {
        let status = self.network().network_status().await.map_err(RethError::other)?;
        Ok(U64::from(status.protocol_version))
    }

    /// Returns the chain id
    fn chain_id(&self) -> U64 {
        U64::from(self.network().chain_id())
    }

    /// Returns the current info for the chain
    fn chain_info(&self) -> RethResult<ChainInfo> {
        Ok(self.provider().chain_info()?)
    }

    fn accounts(&self) -> Vec<Address> {
        self.inner.signers().read().iter().flat_map(|s| s.accounts()).collect()
    }

    fn is_syncing(&self) -> bool {
        self.network().is_syncing()
    }

    /// Returns the [`SyncStatus`] of the network
    fn sync_status(&self) -> RethResult<SyncStatus> {
        let status = if self.is_syncing() {
            let current_block = U256::from(
                self.provider().chain_info().map(|info| info.best_number).unwrap_or_default(),
            );
            SyncStatus::Info(SyncInfo {
                starting_block: self.inner.starting_block(),
                current_block,
                highest_block: current_block,
                warp_chunks_amount: None,
                warp_chunks_processed: None,
            })
        } else {
            SyncStatus::None
        };
        Ok(status)
    }

    fn chain_spec(&self) -> Arc<ChainSpec> {
        self.inner.provider().chain_spec()
    }

    /// Replays all transactions in a block
    async fn get_trx_trace(&self, block_number: BlockNumberOrTag) -> EthResult<Option<Vec<TraceResultsWithTransactionHash>>>{
        
        let tracer_clone = self.tracer.clone();
        let trace_set:HashSet<TraceType> = HashSet::from([TraceType::Trace]);

        tracer_clone.unwrap().replay_block_transactions( BlockId::Number(block_number), trace_set).await
    }

    //Returns SealedBlock by id
    async fn get_block_by_id(&self, block_id: BlockId) -> EthResult<Option<SealedBlock>>{
        let tracer_clone = self.tracer.clone();

        tracer_clone.unwrap().eth_api().block(block_id).await
    }

    /// Returns author and uncle rewards at a given block.
    async fn get_block_rewards(&self, block:&SealedBlock) -> EthResult<Option<Vec<LocalizedTransactionTrace>>>{
        let tracer_clone = self.tracer.clone();

        tracer_clone.unwrap().get_block_rewards(block).await
    }
}
