use subxt::{OnlineClient, PolkadotConfig, utils::{AccountId32}};
use subxt_signer::sr25519::dev;

#[subxt::subxt(
    runtime_metadata_path = "./metadata.scale")]
pub mod rococo {}

type AccountData = rococo::runtime_types::pallet_balances::types::AccountData<::core::primitive::u128>;

async fn check_balance(address: AccountId32, api: OnlineClient::<PolkadotConfig>) -> Option<AccountData> {

    let balance_query = rococo::storage().balances().account(address);
    let balance = api.storage().at_latest().await.unwrap().fetch(&balance_query).await.unwrap();

    return balance;

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core_to_renew: u16 = 1;
    let alice = dev::alice();

    const CORETIME_URI: &str = "ws://127.0.0.1:9920";

    let coretime_api = OnlineClient::<PolkadotConfig>::from_url(CORETIME_URI).await.unwrap();
    let constant_query = rococo::constants().balances().existential_deposit();
    let existential_deposit = coretime_api.constants().at(&constant_query)?;

    let mut blocks_sub = coretime_api.blocks().subscribe_finalized().await?;

    while let Some(block) = blocks_sub.next().await {
        let block = block?;

        let block_number = block.header().number;

        let events = block.events().await?;

        for event in events.find::<rococo::broker::events::Renewable>() {
            let free_balance = check_balance(alice.public_key.into().clone(), coretime_api.clone()).await.unwrap().free;

            let evt = event?;

            let price = evt.price;
            let core = evt.core;

            if core == core_to_renew && free_balance >= ( price + existential_deposit ) {
                let renewal_tx = rococo::tx().broker().renew(core);

                coretime_api.tx().sign_and_submit_then_watch_default(&renewal_tx, &alice)
                .await?
                .wait_for_finalized_success()
                .await?;
                
                println!("Block #{block_number}");
                println!("Core #{core} renewed by {alice}");

            }

        }
    
    }

    Ok(())
}
