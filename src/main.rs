mod ipfs;
mod project;
mod round;

use crate::ipfs::ipfs_get;
use crate::round::index_round_factory;
use ethers::abi::ParamType::{Address as ParamAddress, Uint as AbiUint};
use ethers::abi::{decode, Uint};
use ethers::prelude::*;
use ethers::types::Chain::Optimism;
use ethers::{
    core::types::{Address, Bytes, Filter},
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use sea_orm::{Database, DatabaseConnection};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
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

const DB_URL: &str = env!("DATABASE_URL");

#[derive(Serialize, Deserialize, Debug)]
pub struct IpfsValue {
    hash: String,
    value: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DB_URL)
        .await?;

    let mut provider = Provider::<Http>::try_from(HTTP_URL)?;
    provider.set_chain(Optimism);
    let client = Arc::new(provider);

    let value =
        ipfs_get(
            pool,
            "bafkreih2t2iwmgr7lqghhvtlntfwfxarqpzja3f44oer7gndmfd72gculi".into(),
        )
        .await;

    dbg!(value);

    // index_round_factory(&client, "0x04E753cFB8c8D1D7f776f7d7A033740961b6AEC2".parse::<Address>()?, None).await?;

    // let filter = Filter::new()
    //     .address(ROUND_ADDRESS.parse::<Address>()?)
    //     .event("Voted(address,uint256,address,address,bytes32,address)")
    //     .from_block(16530241);
    // let logs = client.get_logs(&filter).await?;
    // println!("{} votes found!", logs.iter().len());
    //
    // let mut votes = vec![];
    // for log in logs.iter() {
    //     let types = vec![ParamAddress, AbiUint(256), ParamAddress];
    //     let decoded = decode(&types, &log.data);
    //     if let Ok(decoded) = decoded {
    //         let vote = Vote {
    //             token: decoded[0].clone().into_address().unwrap_or_default(),
    //             amount: decoded[1].clone().into_uint().unwrap_or_default(),
    //             voter: Address::from(log.topics[1]),
    //             grant_address: decoded[2].clone().into_address().unwrap_or_default(),
    //             project_id: Bytes::from(log.topics[2].as_bytes().to_vec()),
    //             round_address: Address::from(log.topics[3]),
    //         };
    //         votes.push(vote);
    //     };
    // }
    //
    // dbg!(votes);

    Ok(())
}
