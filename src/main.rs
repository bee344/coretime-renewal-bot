use subxt::{OnlineClient, PolkadotConfig, utils::{AccountId32}};
use subxt_signer::sr25519::dev;

#[subxt::subxt(
    runtime_metadata_path = "./metadata.scale")]
pub mod rococo {}

type AccountData = rococo::runtime_types::pallet_balances::types::AccountData<::core::primitive::u128>;
type AccountInfo = rococo::runtime_types::frame_system::AccountInfo<::core::primitive::u32, AccountData>;
type StatusRecord = rococo::runtime_types::pallet_broker::types::StatusRecord;

async fn check_balance(address: AccountId32, api: OnlineClient::<PolkadotConfig>) -> Option<AccountInfo> {

    let balance_query = rococo::storage().system().account(address);
    let balance = api.storage().at_latest().await.unwrap().fetch(&balance_query).await.unwrap();

    return balance;

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core_to_renew: u16 = 1;
    let alice = dev::alice();

    let address: AccountId32 = alice.public_key().into();

    const CORETIME_URI: &str = "ws://127.0.0.1:9910";

    let coretime_api = OnlineClient::<PolkadotConfig>::from_url(CORETIME_URI).await.unwrap();
    let constant_query = rococo::constants().balances().existential_deposit();
    let existential_deposit = coretime_api.constants().at(&constant_query)?;

    let mut blocks_sub = coretime_api.blocks().subscribe_finalized().await?;

    let mut renewable = false;
    let mut price: u128 = 0;
    let mut core: u16 = 0;
    let mut begin: u32 = 0;

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
            }
        }

        if renewable {
            let sale_info = coretime_api.storage().at_latest().await.unwrap().fetch(&rococo::storage().broker().sale_info()).await.unwrap();

            if sale_info.unwrap().region_begin == begin {
                let free_balance = check_balance(address.clone(), coretime_api.clone()).await.unwrap().data.free;

                if free_balance >= ( price + existential_deposit ) {
                    let tx_config = DefaultExtrinsicParamsBuilder::<PolkadotConfig>::new()
                    .build();

                    let renewal_tx = rococo::tx().broker().renew(core);

                    coretime_api.tx().sign_and_submit_then_watch_default(&renewal_tx, &alice)
                    .await?
                    .wait_for_finalized_success()
                    .await?;

                }
            }
            
        }
    
    }

    Ok(())
}
