# coretime-renewal-bot

This is a bot to keep track of when a core becomes renewable, and when it does,
renew it using the balance of `alice`.

For this, it checks the Coretime Chain's blocks for the desired core's `Renewable`
event, which indicates the core became renewable, then checks that the balance 
of the account has enough funds for the renewal and calls `broker.renew(core)`.

### Testing

For testing, we use [`zombienet v1.3.102`](https://github.com/paritytech/zombienet/tree/v1.3.102)
with the `polkadot`, `polkadot-execute-worker` and `polkadot-prepare-worker` with
`--features fast-runtime`, and `polkadot-parachain`,
built from source ([tested with v1.10.0](https://github.com/paritytech/polkadot-sdk/tree/polkadot-v1.10.0)).

We also use the `subxt cli`. For installing you can run:
```bash
cargo install subxt-cli
```

Then we need to run the following command to retrieve the metadata from the `coretime rococo` parachain:
```bash
subxt metadata --url https://rococo-coretime-rpc.polkadot.io -f bytes > metadata.scale
```
After this was setup, we need to start the `zombienet` with:
```bash
./zombienet-linux-x64 -p native spawn ./config/coretime-network.toml
```
Wait for it to be up and then open a new terminal and run:
```bash
yarn start
```
This will setup the cores and tasks. Once those txs are done, we can run the bot with:

```bash
cargo run
```
