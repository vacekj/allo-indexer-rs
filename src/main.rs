use ethers::{
    core::types::{Address, Filter},
    providers::{Http, Middleware, Provider},
};
use eyre::Result;
use std::sync::Arc;
use ethers::abi::{decode, Uint};
use ethers::abi::ethabi::Bytes;
use ethers::abi::ParamType::{Address as ParamAddress, Uint as AbiUint};
use ethers::types::H256;

const HTTP_URL: &str = "https://rpc.flashbots.net";
const ROUND_ADDRESS: &str = "0x746b951FA10a89d6cbe70d4EE23531f907B58Bc0";

#[derive(Debug)]
struct Vote {
    token: Address,
    amount: Uint,
    voter: Address,
    grant_address: Address,
    project_id: Bytes,
    round_address: Address,
}

#[tokio::main]
async fn main() -> Result<()> {
    let provider = Provider::<Http>::try_from(HTTP_URL)?;
    let client = Arc::new(provider);
    let filter = Filter::new()
        .address(ROUND_ADDRESS.parse::<Address>()?)
        .event("Voted(address,uint256,address,address,bytes32,address)")
        .from_block(16530241);
    let logs = client.get_logs(&filter).await?;
    println!("{} votes found!", logs.iter().len());

    let mut votes = vec![];
    for log in logs.iter() {
        let types = vec![ParamAddress, AbiUint(256), ParamAddress];
        let decoded = decode(&types, &log.data);
        if let Ok(decoded) = decoded {
            let vote = Vote {
                token: decoded[0].clone().into_address().unwrap_or_default(),
                amount: decoded[1].clone().into_uint().unwrap_or_default(),
                voter: Default::default(),
                grant_address: decoded[2].clone().into_address().unwrap_or_default(),
                project_id: vec![],
                round_address: Default::default(),
            };
            votes.push(vote);
        };
    }

    dbg!(votes);

    Ok(())
}
