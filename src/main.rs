use subxt::{OnlineClient, PolkadotConfig, utils::{AccountId32}};
use subxt_signer::sr25519::dev::{self};

#[subxt::subxt(
    runtime_metadata_path = "./metadata.scale")]
pub mod polkadot {}

async fn check_balance(address: AccountId32, api: OnlineClient::<PolkadotConfig>) -> Option<polkadot::runtime_types::pallet_balances::types::AccountData<::core::primitive::u128>> {

    let balance_query = polkadot::storage().balances().account(address);
    let balance = api.storage().at_latest().await.unwrap().fetch(&balance_query).await.unwrap();

    return balance;

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core_to_renew: u16 = 1;
    let bob: AccountId32 = dev::bob().public_key().into();
    let alice = dev::alice();

    const CORETIME_URI: &str = "ws://127.0.0.1:9920";

    let coretime_api = OnlineClient::<PolkadotConfig>::from_url(CORETIME_URI).await.unwrap();
    let constant_query = polkadot::constants().balances().existential_deposit();
    let existential_deposit = coretime_api.constants().at(&constant_query)?;

    let mut blocks_sub = coretime_api.blocks().subscribe_finalized().await?;

    while let Some(block) = blocks_sub.next().await {
        let block = block?;

        let block_number = block.header().number;

        let events = block.events().await?;

        for event in events.find::<polkadot::broker::events::Renewable>() {
            let free_balance = check_balance(bob.clone(), coretime_api.clone()).await.unwrap().free;

            let evt = event?;

            let price = evt.price;
            let core = evt.core;

            if core == core_to_renew && free_balance >= ( price + existential_deposit ) {
                let renewal_tx = polkadot::tx().broker().renew(core);

                coretime_api.tx().sign_and_submit_then_watch_default(&renewal_tx, &alice)
                .await?
                .wait_for_finalized_success()
                .await?;
                
                println!("Block #{block_number}");
                println!("Core #{core} renewed by {bob}");

            }

        }
    
    }

    Ok(())
}
