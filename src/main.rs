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
type AllowedRenewalId = rococo::runtime_types::pallet_broker::types::AllowedRenewalId;

// Retrieve the balance of an account
async fn check_balance(address: AccountId32, api: OnlineClient::<PolkadotConfig>) -> Option<AccountInfo> {

    let balance_query = rococo::storage().system().account(address);
    let balance = api.storage().at_latest().await.unwrap().fetch(&balance_query).await.unwrap();

    return balance;
}

// Retrieve the information of a Coretime Sale
async fn get_sale_info(api: OnlineClient::<PolkadotConfig>) -> Option<SaleInfoRecord> {
    let info_query = rococo::storage().broker().sale_info();
    let info = api.storage().at_latest().await.unwrap().fetch(&info_query).await.unwrap();

    return info;
}

// Retrieve the Coretime Broker's configuration, in order to retrieve the renewal privilege period
async fn get_broker_config(api:OnlineClient::<PolkadotConfig>) -> Option<ConfigRecord> {
    let config_query = rococo::storage().broker().configuration();
    let broker_config = api.storage().at_latest().await.unwrap().fetch(&config_query).await.unwrap();

    return broker_config;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // This is a dev account to exemplify how the bot would behave, but in production
    // it would require an actual keypair.
    let signer = dev::alice();
    let address: AccountId32 = signer.public_key().into();

    // URL of the Coretime RPC Node to which we will connect
    let uri: &str = &args[1];

    // ID of the core we want to keep watch of
    let string_to_renew: &str = &args[2];
    let core_to_renew: u16 = string_to_renew.parse::<u16>().expect("Invalid core ID.");

    let mut renewable = false;
    let mut price: u128 = 0;
    let mut core: u16 = 0;
    let mut begin: u32 = 0;
    let mut renewal_start: u32 = 0;


    let coretime_api = OnlineClient::<PolkadotConfig>::from_url(uri).await.unwrap();

    let constant_query = rococo::constants().balances().existential_deposit();
    let existential_deposit = coretime_api.constants().at(&constant_query)?;

    // The interlude_length is the period in Blocks during which the owner of the
    // ending lease has preference to renew the core
    let interlude_length = get_broker_config(coretime_api.clone()).await.unwrap().interlude_length;

    let mut blocks_sub = coretime_api.blocks().subscribe_finalized().await?;

    while let Some(block) = blocks_sub.next().await {
        let block = block?;

        let block_number = block.header().number;

        let events = block.events().await?;

        for event in events.find::<rococo::broker::events::Renewable>() {
            let evt = event?;

            core = evt.core;

            // Check if the core that became renewable is the one we are looking for,
            // and if it is, retrieve the data
            if core == core_to_renew {
                price = evt.price;
                begin = evt.begin;
                renewable = true;
                println!("Core is renewable from region {begin} at the price of {price}.");


                let renewal_id = {AllowedRenewalId {core: core.clone(), when:  begin.clone()}}; 
                let storage_query = rococo::storage().broker().allowed_renewals(renewal_id);
                let allowed_renewals = coretime_api.storage().at_latest().await.unwrap().fetch(&storage_query).await.unwrap().unwrap();

                println!("Allowed renewals info:\n{allowed_renewals:?}");
            }
        }

        if renewable {
            // Check the funds of the account
            let free_balance = check_balance(address.clone(), coretime_api.clone()).await.unwrap().data.free;

            // Check if the account has enough funds to renew the core and not be reaped
            let enough_funds = free_balance >= ( price + existential_deposit );
            
            // Warning to fund the account
            if !enough_funds {
                println!("Funds not enough to renew the core, please add more funds to your account.");
            }

            // Get info on the upcoming Coretime sale
            let sale_info = get_sale_info(coretime_api.clone()).await.unwrap();

            // Set the block when the renewal period began
            if sale_info.region_begin == begin {
                renewal_start = block_number;
            }

            // Check if the sale has begun
            if sale_info.region_begin >= begin {
                // If we are in the sale period, check if we are in the renewal 
                // window and if we have enough funds, then proceed with the renewal
                if ( renewal_start + interlude_length ) >= block_number && enough_funds {
                    println!("In renewal window, attempting to renew core #{core_to_renew}.");
                    let renewal_tx = rococo::tx().broker().renew(core);
    
                    coretime_api.tx().sign_and_submit_then_watch_default(&renewal_tx, &signer)
                    .await?
                    .wait_for_finalized_success()
                    .await?;
    
                    println!("Renewed core #{core} on block #{block_number}.");

                    // Reset renewable, since we already renew our lease    
                    renewable = false;
                // Check if we have no funds but if we are in the renewal window
                } else if ( renewal_start + interlude_length ) >= block_number {
                    println!("Funds not enough to renew the core, please add more funds to your account.");
                // If previous checks fail, it means the core has to be purchased through the open market
                // We also reset renewable, since the renewal window has passed
                } else {
                    println!("Outside of renewal window. Coretime must be purchased on the open market.");

                    renewable = false;
                }
            }
        }
    }

    Ok(())
}
