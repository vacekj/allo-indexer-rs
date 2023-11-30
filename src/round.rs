use chrono::{DateTime, TimeZone, Utc};
use ethers::prelude::*;
use eyre::Result;
use futures::future;
use futures::future::join_all;
use rayon::prelude::*;
use std::sync::Arc;
use tokio::task;

abigen!(RoundImplementation, "./abis/RoundImplementation.json",);

#[derive(Debug)]
pub struct Round {
    // application_meta_ptr: (U256, String),
    // applications_start_time: DateTime<Utc>,
    // applications_end_time: DateTime<Utc>,
    // match_amount: U256,
    id: Address,
    payout_strategy: Address,
    voting_strategy: Address,
    // round_meta_ptr: (U256, String),
    token: Address,
    // round_start_time: DateTime<Utc>,
    // round_end_time: DateTime<Utc>,
}

pub async fn index_round_factory(
    provider: &Arc<Provider<Http>>,
    round_factory_address: Address,
    from_block: Option<u64>,
) -> Result<Vec<Round>> {
    let filter = Filter::new()
        .address(round_factory_address)
        .event("RoundCreated(address,address,address)")
        .from_block(from_block.unwrap_or(0));

    let logs = provider.get_logs(&filter).await?;

    let rounds_addresses: Vec<Address> = logs
        .iter()
        .map(|log| Address::from(log.topics[1]))
        .collect();

    let mut futures = Vec::new();

    for round in rounds_addresses {
        let provider_clone = Arc::clone(&provider);
        let future = task::spawn(async move { index_round(provider_clone, &round.clone()).await });
        futures.push(future);
    }

    let results = join_all(futures)
        .await
        .into_iter()
        .map(|val| val.unwrap())
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    println!(
        "indexed {} rounds on chain {}",
        results.len(),
        provider.get_chainid().await?
    );

    Ok(results)
}

pub async fn index_round(provider: Arc<Provider<Http>>, round_address: &Address) -> Result<Round> {
    let contract = RoundImplementation::new(*round_address, provider.to_owned());
    let token = contract.token().await?;
    // let application_meta_ptr = contract.application_meta_ptr().await?;
    // let applications_end_time_timestamp = contract.applications_end_time().await?;
    // let applications_end_time = DateTime::from_timestamp(i64::try_from(applications_end_time_timestamp).unwrap(), 0).unwrap();
    // let applications_start_time_timestamp = contract.applications_start_time().await?;
    // let applications_start_time = DateTime::from_timestamp(i64::try_from(applications_start_time_timestamp).unwrap(), 0).unwrap();
    // let match_amount = contract.match_amount().await?;
    // let round_meta_ptr = contract.round_meta_ptr().await?;
    let payout_strategy = contract.payout_strategy().await?;
    let voting_strategy = contract.voting_strategy().await?;
    // let round_end_time_timestamp = contract.round_end_time().await?;
    // let round_end_time = DateTime::from_timestamp(i64::try_from(round_end_time_timestamp).unwrap(), 0).unwrap();
    // let round_start_time_timestamp = contract.round_start_time().await?;
    // let round_start_time = DateTime::from_timestamp(i64::try_from(round_start_time_timestamp).unwrap(), 0).unwrap();

    let round = Round {
        // application_meta_ptr,
        // applications_end_time,
        // applications_start_time,
        // match_amount,
        id: *round_address,
        payout_strategy,
        voting_strategy,
        // round_meta_ptr,
        token,
        // round_start_time,
        // round_end_time,
    };

    Ok(round)
}
