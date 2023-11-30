mod ipfs;
mod project;
mod round;

// use crate::ipfs::ipfs_get;
use crate::round::index_round_factory;
use ethers::abi::Uint;
use ethers::prelude::*;
use ethers::types::Chain::Optimism;
use ethers::{
    core::types::{Address, Bytes},
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

const HTTP_URL: &str = "https://opt-mainnet.g.alchemy.com/v2/BUyiIrMBAy0UGmIASaEopcSET3Ce_Tb_";

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

#[tokio::main]
async fn main() -> Result<()> {
    let mut provider = Provider::<Http>::try_from(HTTP_URL)?;
    provider.set_chain(Optimism);
    let client = Arc::new(provider);
    index_round_factory(
        &client,
        "0x04E753cFB8c8D1D7f776f7d7A033740961b6AEC2".parse::<Address>()?,
        None,
    )
    .await?;

    Ok(())
}
