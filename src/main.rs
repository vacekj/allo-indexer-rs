mod ipfs;
mod project;
mod round;

use std::ops::BitAnd;
use std::str::FromStr;
// use crate::ipfs::ipfs_get;
use crate::round::index_round_factory;
use ethers::abi::Uint;
use ethers::prelude::*;
use ethers::types::Chain::{Mainnet, Optimism};
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
            round_factory_address: Address::from_str("0x9Cb7f434aD3250d1656854A9eC7A71EceC6eE1EF")
                .unwrap(),
            url: "https://mainnet.infura.io/v3/dca3b3f8ffa84b8c99987faf694cde1f",
            chain: Mainnet,
        },
        ChainDef {
            round_factory_address: Address::from_str("0x9Cb7f434aD3250d1656854A9eC7A71EceC6eE1EF")
                .unwrap(),
            url: "https://mainnet.infura.io/v3/dca3b3f8ffa84b8c99987faf694cde1f",
            chain: Mainnet,
        },
        ChainDef {
            round_factory_address: Address::from_str("0x9Cb7f434aD3250d1656854A9eC7A71EceC6eE1EF")
                .unwrap(),
            url: "https://mainnet.infura.io/v3/dca3b3f8ffa84b8c99987faf694cde1f",
            chain: Mainnet,
        },
        ChainDef {
            round_factory_address: Address::from_str("0x9Cb7f434aD3250d1656854A9eC7A71EceC6eE1EF")
                .unwrap(),
            url: "https://mainnet.infura.io/v3/dca3b3f8ffa84b8c99987faf694cde1f",
            chain: Mainnet,
        },
        ChainDef {
            round_factory_address: Address::from_str("0x9Cb7f434aD3250d1656854A9eC7A71EceC6eE1EF")
                .unwrap(),
            url: "https://mainnet.infura.io/v3/dca3b3f8ffa84b8c99987faf694cde1f",
            chain: Mainnet,
        },
    ];

    let handles: Vec<_> = chains
        .iter()
        .map(|obj| {
            let chain_def_clone = obj.clone(); // Clone ChainDef for the task
            task::spawn(async move {
                index_chain(&chain_def_clone).await;
            })
        })
        .collect();

    let _results = futures::future::join_all(handles).await;

    Ok(())
}

async fn index_chain(chain_def: &ChainDef) -> Result<()> {
    let mut provider = Provider::<Http>::try_from(chain_def.url)?;
    provider.set_chain(Optimism);
    let client = Arc::new(provider);
    index_round_factory(&client, chain_def.round_factory_address, None).await?;

    Ok(())
}
