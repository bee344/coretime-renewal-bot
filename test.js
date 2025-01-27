const { Keyring } = require('@polkadot/keyring');
const { ApiPromise, WsProvider } = require('@polkadot/api');
const { cryptoWaitReady } = require('@polkadot/util-crypto');

async function main() {
    const relayWsProvider = new WsProvider("ws://127.0.0.1:9900");
    const coretimeWsProvider = new WsProvider("ws://127.0.0.1:9910");
    const collatorWsProvider = new WsProvider("ws://127.0.0.1:9920");

    const collatorApi = await ApiPromise.create({
        provider: collatorWsProvider,
    })
    await collatorApi.isReady;

    const genesisHeader = await collatorApi.rpc.chain.getHeader();
    const validationCode = await collatorApi.rpc.state.getStorage("0x3A636F6465");

    await cryptoWaitReady();

    const keyring = new Keyring({ type: "sr25519" });
    const alice = keyring.addFromUri("//Alice");

    const relayApi = await ApiPromise.create({
        provider: relayWsProvider,
    });
    await relayApi.isReady;

    const relayCalls = [
        relayApi.tx.configuration.setCoretimeCores({ new: 1 }),
        relayApi.tx.coretime.assignCore(0, 20, [[{ task: 1005 }, 57600]], null),
        relayApi.tx.registrar.forceRegister(
            alice.address,
            0,
            100,
            genesisHeader.toHex(),
            validationCode.toHex(),
        )
    ];
    const relaySudoBatch = relayApi.tx.sudo.sudo(relayApi.tx.utility.batch(relayCalls));

    await new Promise(async (resolve, reject) => {
        const unsub = await relaySudoBatch.signAndSend(alice, (result) => {
            console.log(`Current status is ${result.status}`);
            if (result.status.isInBlock) {
                console.log(
                    `Transaction included at blockHash ${result.status.asInBlock}`
                );
            } else if (result.status.isFinalized) {
                console.log(
                    `Transaction finalized at blockHash ${result.status.asFinalized}`
                );
                unsub();
                return resolve();
            } else if (result.isError) {
                console.log(`Transaction Error`);
                unsub();
                return reject();
            }
        });
    });

    const coretimeApi = await ApiPromise.create({
        provider: coretimeWsProvider,
    });
    await coretimeApi.isReady;

    const coretimeCalls = [
        // Default broker configuration
        coretimeApi.tx.broker.configure({
            advanceNotice: 5,
            interludeLength: 2,
            leadinLength: 1,
            regionLength: 2,
            idealBulkProportion: 100,
            limitCoresOffered: null,
            renewalBump: 500,
            contributionTimeout: 5,
        }),
        // We need MOARE cores.
        coretimeApi.tx.broker.requestCoreCount(2),
        // Set a lease for the broker chain itself.
        coretimeApi.tx.broker.setLease(
            1005,
            1000,
        ),
        // Set a lease for parachain 100
        coretimeApi.tx.broker.setLease(
            100,
            10,
        ),
        // Start sale to make the broker "work", but we don't offer any cores
        // as we have fixed leases only anyway.
        coretimeApi.tx.broker.startSales(1, 0),
    ];
    const coretimeSudoBatch = coretimeApi.tx.sudo.sudo(coretimeApi.tx.utility.batch(coretimeCalls));

    await new Promise(async (resolve, reject) => {
        const unsub = await coretimeSudoBatch.signAndSend(alice, (result) => {
            console.log(`Current status is ${result.status}`);
            if (result.status.isInBlock) {
                console.log(
                    `Transaction included at blockHash ${result.status.asInBlock}`
                );
            } else if (result.status.isFinalized) {
                console.log(
                    `Transaction finalized at blockHash ${result.status.asFinalized}`
                );
                unsub();
                return resolve();
            } else if (result.isError) {
                console.log(`Transaction error`);
                unsub();
                return resolve();
            }
        });
    });

}

main()
    .catch(console.error)
    .finally(() => process.exit());
