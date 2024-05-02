# coretime-renewal-bot

This is a bot to keep track of when a core becomes renewable, and when it does,
renew it.

For this, it checks the Coretime Chain's blocks for the desired core's `Renewable`
event, which indicates the core became renewable, then checks that the balance 
of the account has enough funds for the renewal and calls `broker.renew(core)`.
It also takes into account whether we are on the renewal period or not, and indicates
if the core was renewed or if it hasn't, whether it was due to lack of funds or because
the renewal window was missed and we have to purchase Coretime through the open market.

**DISCLAIMER**
This code is designed as a guide on how to handle a Core's coretime renewal, and
is not ready for production. For it to be usable on a live chain, the signer and 
endpoint must be changed.

### Running the example

For running the example and seeing its behaviour, we use [`zombienet v1.3.102`](https://github.com/paritytech/zombienet/tree/v1.3.102)
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
cargo run <URL> <CORE_NUMBER>
```
