#![allow(unused_imports, dead_code)]

mod ipfs;
mod project;
mod round;

use std::ops::BitAnd;
use std::str::FromStr;
// use crate::ipfs::ipfs_get;
use crate::round::{index_round_factory, Round};
use ethers::abi::Uint;
use ethers::prelude::*;
use ethers::types::Chain::{Arbitrum, Fantom, Mainnet, Optimism, Polygon};
use ethers::{
    core::types::{Address, Bytes},
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use futures::stream::iter;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{join, task};

#[derive(Debug)]
struct Vote {
    token: Address,
    amount: Uint,
    voter: Address,
    grant_address: Address,
    project_id: Bytes,
    round_address: Address,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IpfsValue {
    hash: String,
    value: String,
}

#[derive(Clone)]
struct ChainDef {
    chain: Chain,
    url: &'static str,
    round_factory_address: Address,
}

#[tokio::main]
async fn main() -> Result<()> {
    let chains: Vec<ChainDef> = vec![
        ChainDef {
            round_factory_address: Address::from_str("0x04E753cFB8c8D1D7f776f7d7A033740961b6AEC2")
                .unwrap(),
            url: "https://opt-mainnet.g.alchemy.com/v2/BUyiIrMBAy0UGmIASaEopcSET3Ce_Tb_",
            chain: Optimism,
        },
        ChainDef {
            round_factory_address: Address::from_str("0x9Cb7f434aD3250d1656854A9eC7A71EceC6eE1EF")
                .unwrap(),
            url: "https://mainnet.infura.io/v3/dca3b3f8ffa84b8c99987faf694cde1f",
            chain: Mainnet,
        },
        ChainDef {
            round_factory_address: Address::from_str("0x5ab68dCdcA37A1C2b09c5218e28eB0d9cc3FEb03")
                .unwrap(),
            url: "https://polygon-mainnet.infura.io/v3/dca3b3f8ffa84b8c99987faf694cde1f",
            chain: Polygon,
        },
        ChainDef {
            round_factory_address: Address::from_str("0xfb08d1fD3a7c693677eB096E722ABf4Ae63B0B95")
                .unwrap(),
            url: "https://rpc3.fantom.network",
            chain: Fantom,
        },
        ChainDef {
            round_factory_address: Address::from_str("0xF2a07728107B04266015E67b1468cA0a536956C8")
                .unwrap(),
            url: "https://arbitrum.llamarpc.com",
            chain: Arbitrum,
        },
        ChainDef {
            round_factory_address: Address::from_str("0x8AdFcF226dfb2fA73788Ad711C958Ba251369cb3")
                .unwrap(),
            url: "https://rpc.publicgoods.network",
            chain: Mainnet,
        },
    ];

    let handles: Vec<_> =
        chains
            .iter()
            .map(|obj| {
                let chain_def_clone = obj.clone(); // Clone ChainDef for the task
                task::spawn(async move { index_chain(&chain_def_clone).await.unwrap() })
            })
            .collect();

    let rounds = futures::future::join_all(handles).await;
    dbg!(rounds);
    Ok(())
}

async fn index_chain(chain_def: &ChainDef) -> Result<Vec<Round>> {
    let mut provider = Provider::<Http>::try_from(chain_def.url)?;
    provider.set_chain(Optimism);
    let client = Arc::new(provider);
    let rounds = index_round_factory(&client, chain_def.round_factory_address, None).await?;

    Ok(rounds)
}
