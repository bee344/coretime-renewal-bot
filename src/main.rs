use subxt::{OnlineClient, PolkadotConfig, utils::{AccountId32}};
use subxt_signer::sr25519::dev;
use std::env;

#[subxt::subxt(
    runtime_metadata_path = "./metadata.scale")]
pub mod rococo {}

type AccountData = rococo::runtime_types::pallet_balances::types::AccountData<u128>;
type AccountInfo = rococo::runtime_types::frame_system::AccountInfo<u32, AccountData>;
type SaleInfoRecord = rococo::runtime_types::pallet_broker::types::SaleInfoRecord<u128, u32>;
type ConfigRecord = rococo::runtime_types::pallet_broker::types::ConfigRecord<u32, u32>;

async fn check_balance(address: AccountId32, api: OnlineClient::<PolkadotConfig>) -> Option<AccountInfo> {

    let balance_query = rococo::storage().system().account(address);
    let balance = api.storage().at_latest().await.unwrap().fetch(&balance_query).await.unwrap();

    return balance;
}

async fn get_sale_info(api: OnlineClient::<PolkadotConfig>) -> Option<SaleInfoRecord> {
    let info = api.storage().at_latest().await.unwrap().fetch(&rococo::storage().broker().sale_info()).await.unwrap();

    return info;
}

async fn get_broker_config(api:OnlineClient::<PolkadotConfig>) -> Option<ConfigRecord> {
    let broker_config = api.storage().at_latest().await.unwrap().fetch(&rococo::storage().broker().configuration()).await.unwrap();

    return broker_config;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let alice = dev::alice();

    let address: AccountId32 = alice.public_key().into();

    let uri: &str = &args[1];

    let string_to_renew: &str = &args[2];
    let core_to_renew: u16 = string_to_renew.parse::<u16>().expect("Invalid core ID.");


    let mut renewable = false;
    let mut price: u128 = 0;
    let mut core: u16 = 0;
    let mut begin: u32 = 0;
    let mut renewal_start: u32 = 0;
    let coretime_api = OnlineClient::<PolkadotConfig>::from_url(uri).await.unwrap();
    let constant_query = rococo::constants().balances().existential_deposit();
    let interlude_length = get_broker_config(coretime_api.clone()).await.unwrap().interlude_length;
    let existential_deposit = coretime_api.constants().at(&constant_query)?;

    let mut blocks_sub = coretime_api.blocks().subscribe_finalized().await?;

    while let Some(block) = blocks_sub.next().await {
        let block = block?;

        let block_number = block.header().number;

        let events = block.events().await?;

        for event in events.find::<rococo::broker::events::Renewable>() {
            let evt = event?;

            core = evt.core;

            if core == core_to_renew {
                price = evt.price;
                begin = evt.begin;
                renewable = true;
                println!("Core is renewable from region {begin} at the price of {price}.");
            }
        }

        if renewable {
            let free_balance = check_balance(address.clone(), coretime_api.clone()).await.unwrap().data.free;

            let enough_funds = free_balance >= ( price + existential_deposit );
            
            if !enough_funds {
                println!("Funds not enough to renew the core, please add more funds to your account.");
                continue
            }

            let sale_info = get_sale_info(coretime_api.clone()).await.unwrap();

            if sale_info.region_begin == begin {
                renewal_start = block_number;
            }

            if sale_info.region_begin >= begin && ( renewal_start + interlude_length ) <= block_number {
                println!("In renewal window, attempting to renew core #{core_to_renew}.");
                let renewal_tx = rococo::tx().broker().renew(core);

                coretime_api.tx().sign_and_submit_then_watch_default(&renewal_tx, &alice)
                .await?
                .wait_for_finalized_success()
                .await?;

                println!("Renewed core #{core} on block #{block_number}.");

                renewable = false;
            }

        }
    }

    Ok(())
}
