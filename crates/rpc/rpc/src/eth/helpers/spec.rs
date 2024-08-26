use jsonrpsee::http_client::HttpClient;
use reth_chainspec::{ChainSpec, EthereumHardforks};
use reth_consensus_common::calc::{base_block_reward, base_block_reward_pre_merge, block_reward, ommer_reward};
use reth_network_api::NetworkInfo;
use reth_primitives::{Header, U256};
use reth_provider::{BlockNumReader, ChainSpecProvider, StageCheckpointReader};
use reth_rpc_eth_api::helpers:: EthApiSpec;
use reth_rpc_types::trace::parity::{LocalizedTransactionTrace, RewardAction, RewardType};
use reth_transaction_pool::TransactionPool;

use crate::{trace::reward_trace, EthApi};

impl<Provider, Pool, Network, EvmConfig> EthApiSpec for EthApi<Provider, Pool, Network, EvmConfig>
where
    Pool: TransactionPool + 'static,
    Provider:
        ChainSpecProvider<ChainSpec = ChainSpec> + BlockNumReader + StageCheckpointReader + 'static,
    Network: NetworkInfo + 'static,
    EvmConfig: Send + Sync,
{
    fn provider(
        &self,
    ) -> impl ChainSpecProvider<ChainSpec = ChainSpec> + BlockNumReader + StageCheckpointReader
    {
        self.inner.provider()
    }

    fn network(&self) -> impl NetworkInfo {
        self.inner.network()
    }

    fn starting_block(&self) -> U256 {
        self.inner.starting_block()
    }

    fn signers(&self) -> &parking_lot::RwLock<Vec<Box<dyn reth_rpc_eth_api::helpers::EthSigner>>> {
        self.inner.signers()
    }

    fn get_rpc_client(&self) -> Option<HttpClient>{
        self.rpc_client.clone()
    }

    fn extract_reward_traces(
        &self,
        header: &Header,
        ommers: &[Header],
        base_block_reward: u128,
    ) -> Vec<LocalizedTransactionTrace> {
        let mut traces = Vec::with_capacity(ommers.len() + 1);

        let block_reward = block_reward(base_block_reward, ommers.len());
        traces.push(reward_trace(
            header,
            RewardAction {
                author: header.beneficiary,
                reward_type: RewardType::Block,
                value: U256::from(block_reward),
            },
        ));

        for uncle in ommers {
            let uncle_reward = ommer_reward(base_block_reward, header.number, uncle.number);
            traces.push(reward_trace(
                header,
                RewardAction {
                    author: uncle.beneficiary,
                    reward_type: RewardType::Uncle,
                    value: U256::from(uncle_reward),
                },
            ));
        }
        traces
    }

    fn calculate_base_block_reward(&self, header: &Header) -> Result<Option<u128>, reth_rpc_server_types::RethRpcModule> {
        let chain_spec = self.provider().chain_spec();
        let is_paris_activated = chain_spec.is_paris_active_at_block(header.number);

        Ok(match is_paris_activated {
            Some(true) => None,
            Some(false) => Some(base_block_reward_pre_merge(&chain_spec, header.number)),
            None => {
                // if Paris hardfork is unknown, we need to fetch the total difficulty at the
                // block's height and check if it is pre-merge to calculate the base block reward
                    base_block_reward(
                        chain_spec.as_ref(),
                        header.number,
                        header.difficulty,
                        U256::ZERO,
                    )
            }
        })
    }

    async fn get_block_rewards(
        &self,
        block_header: &Header, omners: &[Header] )-> Result<Option<Vec<LocalizedTransactionTrace>>, reth_rpc_server_types::RethRpcModule>{ 
            let mut trace_rewards:Vec<LocalizedTransactionTrace> = Vec::new();
 
            if let Some(base_block_reward) = self.calculate_base_block_reward(&block_header)? {
                trace_rewards.extend(self.extract_reward_traces(
                    &block_header,
                    &omners,
                    base_block_reward,
                ));
            }
 
            Ok(Some(trace_rewards))
     }   
}
