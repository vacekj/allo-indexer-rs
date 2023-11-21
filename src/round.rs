use std::sync::Arc;
use chrono::{DateTime, TimeZone, Utc};
use ethers::prelude::*;
use ethers::solc::artifacts::RevertStrings::Default;
use eyre::Result;

abigen!(
        RoundImplementation,
       "./abis/RoundImplementation.json",
    );

#[derive(Debug)]
pub struct Round {
    application_meta_ptr: (U256, String),
    applications_start_time: DateTime<Utc>,
    applications_end_time: DateTime<Utc>,
    match_amount: U256,
    payout_strategy: Address,
    round_meta_ptr: (U256, String),
    token: Address,
    round_start_time: DateTime<Utc>,
    round_end_time: DateTime<Utc>,
}

pub async fn index_round_factory(provider: &Arc<Provider<Http>>, round_factory_address: Address, from_block: Option<u64>) -> Result<()> {
    // let filter = Filter::new()
    //     .address(round_factory_address)
    //     .event("RoundCreated(address,address,address)")
    //     .from_block(from_block.unwrap_or(0));
    //
    // let logs = provider.get_logs(&filter).await?;
    //
    // let mut rounds = vec![];
    // for log in logs.iter() {
    //    let round_address =  Address::from(log.topics[1]);
    //     rounds.push(round_address);
    // }
    //
    // dbg!(rounds);

    index_round(provider, "0x706a4ccb4bd3d0ce60433143a3df9cc8e74970b8".parse::<Address>()?).await;

    Ok(())
}

pub async fn index_round(provider: &Arc<Provider<Http>>, round_address: Address) -> Result<Round> {
    let contract = RoundImplementation::new(round_address, provider.to_owned());
    let token = contract.token().await?;
    let application_meta_ptr = contract.application_meta_ptr().await?;
    let applications_end_time_timestamp = contract.applications_end_time().await?;
    let applications_end_time = DateTime::from_timestamp(i64::try_from(applications_end_time_timestamp).unwrap(), 0).unwrap();
    let applications_start_time_timestamp = contract.applications_start_time().await?;
    let applications_start_time = DateTime::from_timestamp(i64::try_from(applications_start_time_timestamp).unwrap(), 0).unwrap();
    let match_amount = contract.match_amount().await?;
    let round_meta_ptr = contract.round_meta_ptr().await?;
    let payout_strategy = contract.payout_strategy().await?;
    let round_end_time_timestamp = contract.round_end_time().await?;
    let round_end_time = DateTime::from_timestamp(i64::try_from(round_end_time_timestamp).unwrap(), 0).unwrap();
    let round_start_time_timestamp = contract.round_start_time().await?;
    let round_start_time = DateTime::from_timestamp(i64::try_from(round_start_time_timestamp).unwrap(), 0).unwrap();

    let round = Round {
        application_meta_ptr,
        applications_end_time,
        applications_start_time,
        match_amount,
        payout_strategy,
        round_meta_ptr,
        token,
        round_start_time,
        round_end_time,
    };

    dbg!(&round);

    Ok(round)
}