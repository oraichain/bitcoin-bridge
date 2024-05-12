"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const helper_1 = require("./helper");
const connect_1 = require("./connect");
const network_1 = require("./network");
const helper_2 = require("./helper");
const worker_threads_1 = require("worker_threads");
const proto_signing_1 = require("@cosmjs/proto-signing");
const promises_1 = require("timers/promises");
const tx_1 = require("cosmjs-types/cosmos/tx/v1beta1/tx");
const handleMsgSend = async ({ client, clientAddress, sequence, toAddress, }) => {
    try {
        const sendMsg1 = {
            typeUrl: "/cosmos.bank.v1beta1.MsgSend",
            value: {
                fromAddress: clientAddress,
                toAddress, // a sample of to address
                amount: [
                    {
                        denom: network_1.OraiBtcLocalConfig.feeToken,
                        amount: "0",
                    },
                ],
            },
        };
        const txRaw = await client.sign(clientAddress, [sendMsg1], {
            amount: [(0, proto_signing_1.coin)("0", network_1.OraiBtcLocalConfig.feeToken)],
            gas: "100000",
        }, "", {
            accountNumber: 0,
            chainId: network_1.OraiBtcLocalConfig.chainId,
            sequence,
        });
        const txBytes = tx_1.TxRaw.encode(txRaw).finish();
        const txData = await client.broadcastTx(txBytes);
        return txData;
    }
    catch (err) {
        await (0, promises_1.setTimeout)(100);
        return await handleMsgSend({ client, clientAddress, sequence, toAddress });
    }
};
let mappedAddress = {};
const genNewAccount = async () => {
    try {
        const mnemonic = (await proto_signing_1.DirectSecp256k1HdWallet.generate()).mnemonic;
        if (mappedAddress[`${mnemonic}`] !== undefined) {
            throw Error("Generate again");
        }
        console.log(mnemonic);
        mappedAddress[`${mnemonic}`] = 1;
        const { address } = await (0, connect_1.connect)(mnemonic, network_1.OraiBtcLocalConfig, true);
        const derivedAddress = (0, helper_1.deriveAddressByPrefix)(address, network_1.OraiBtcLocalConfig.prefix);
        return [derivedAddress, mnemonic];
    }
    catch (err) {
        console.log("Gen again");
        await (0, promises_1.setTimeout)(100);
        return await genNewAccount();
    }
};
const runService = (workerData) => {
    return new Promise((resolve, reject) => {
        const worker = new worker_threads_1.Worker("./dist/worker.js", {
            workerData,
        });
        worker.on("message", (message) => {
            console.log("Finish one of workers", message);
            resolve(message);
        });
        worker.on("error", reject);
        worker.on("exit", (code) => {
            if (code !== 0) {
                reject(new Error(`Worker stopped with exit code ${code}`));
            }
        });
    });
};
async function main() {
    // get the mnemonic
    const mnemonic = (0, helper_1.getMnemonic)();
    const args = process.argv.slice(2);
    if (!args[0])
        throw Error("Please input number of threads");
    if (!args[1])
        throw Error("Please input number of txs per thread");
    const numberOfThread = parseInt(args[0]);
    const numberOfTxs = parseInt(args[1]);
    if (numberOfThread > 100) {
        throw Error("Threads is too large, may lead to overloaded");
    }
    const { address, client } = await (0, connect_1.connect)(mnemonic, network_1.OraiBtcLocalConfig, true);
    const derivedAddress = (0, helper_1.deriveAddressByPrefix)(address, network_1.OraiBtcLocalConfig.prefix);
    const accountInfo = await (0, helper_2.getAccountInfo)(derivedAddress);
    const sequence = parseInt(accountInfo.result.value.sequence);
    console.log("Init sequence", sequence);
    const array = new Array(numberOfThread).fill(0);
    const addressesFunc = array.map((_, index) => {
        return genNewAccount();
    });
    const addresses = await Promise.all(addressesFunc);
    const initTxs = array.map((_, index) => {
        console.log("Process:", index);
        return handleMsgSend({
            client,
            clientAddress: derivedAddress,
            sequence: sequence + index,
            toAddress: addresses.map((item) => item[0])[index],
        });
    });
    await Promise.all(initTxs);
    console.log("Start handling multi-thread");
    const startTime = new Date().getTime();
    const result = await Promise.all(array.map((x, index) => runService({
        mnemonic: addresses[index][1],
        numOfTxs: numberOfTxs,
    })));
    const endTime = new Date().getTime();
    console.log("Total taked: ", (endTime - startTime) / 1000);
    console.log("Total executed txs: ", result
        .map((item) => item.result)
        .reduce((accumulator, currentValue) => {
        return accumulator + currentValue;
    }, 0)); // Giá trị ban đầu của accumulator là 0
}
main().then(() => {
    process.exit(0);
}, (error) => {
    console.error(error);
    process.exit(1);
});
