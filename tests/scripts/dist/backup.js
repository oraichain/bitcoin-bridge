"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
const helper_1 = require("./helper");
const connect_1 = require("./connect");
const network_1 = require("./network");
const amino_1 = require("@cosmjs/amino");
// import { MsgSend } from "cosmjs-types/cosmos/bank/v1beta1/tx";
const tx_1 = require("cosmjs-types/cosmos/tx/v1beta1/tx");
const helper_2 = require("./helper");
const promises_1 = require("timers/promises");
const proto_signing_1 = require("@cosmjs/proto-signing");
const os_1 = __importDefault(require("os"));
process.env.UV_THREADPOOL_SIZE = os_1.default.cpus().length.toString();
const toAddressConstant = "oraibtc1ehmhqcn8erf3dgavrca69zgp4rtxj5kqzpga4j";
let shouldPrintError = false;
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
            amount: [(0, amino_1.coin)("0", network_1.OraiBtcLocalConfig.feeToken)],
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
        if (shouldPrintError) {
            console.log(err);
        }
        console.log("Retried from ", clientAddress);
        await (0, promises_1.setTimeout)(200);
        await handleMsgSend({ client, clientAddress, sequence, toAddress });
    }
};
const handleExecutePerAccount = async (mnemonic, numberOfTxs) => {
    const { client, address } = await (0, connect_1.connect)(mnemonic, network_1.OraiBtcLocalConfig, true);
    const derivedAddress = (0, helper_1.deriveAddressByPrefix)(address, network_1.OraiBtcLocalConfig.prefix);
    //   console.log(derivedAddress);
    //   const accountInfo = await getAccountInfo(derivedAddress);
    //   const sequence = parseInt(accountInfo.result.value.sequence);
    //   console.log("Sequence: ", sequence);
    // because it is new generated accounts so we don't need to fetch sequence
    const bulkTxs = new Array(numberOfTxs).fill(0).map((_, index) => {
        return handleMsgSend({
            client,
            clientAddress: derivedAddress,
            sequence: 0 + index,
            toAddress: toAddressConstant,
        });
    });
    const data = await Promise.all(bulkTxs);
};
const genNewAccount = async () => {
    try {
        const mnemonic = (await proto_signing_1.DirectSecp256k1HdWallet.generate()).mnemonic;
        const { address } = await (0, connect_1.connect)(mnemonic, network_1.OraiBtcLocalConfig, true);
        const derivedAddress = (0, helper_1.deriveAddressByPrefix)(address, network_1.OraiBtcLocalConfig.prefix);
        return [derivedAddress, mnemonic];
    }
    catch (err) {
        console.log(err);
        await (0, promises_1.setTimeout)(100);
        return await genNewAccount();
    }
};
async function main() {
    // get the mnemonic
    const mnemonic = (0, helper_1.getMnemonic)();
    const args = process.argv.slice(2);
    if (!args[0])
        throw Error("Please input number of accounts");
    if (!args[1])
        throw Error("Please input number of txs per account");
    console.log(`Benchmarking with ${args[0]} accounts and ${args[1]} txs per each`);
    const numberOfAccount = parseInt(args[0]);
    const numberOfTxsPerAccount = parseInt(args[1]);
    const arr = new Array(numberOfAccount).fill(0);
    const addressesFunc = arr.map((_, index) => {
        return genNewAccount();
    });
    const addresses = await Promise.all(addressesFunc);
    const { client, address } = await (0, connect_1.connect)(mnemonic, network_1.OraiBtcLocalConfig, true);
    const derivedAddress = (0, helper_1.deriveAddressByPrefix)(address, network_1.OraiBtcLocalConfig.prefix);
    const accountInfo = await (0, helper_2.getAccountInfo)(derivedAddress);
    const sequence = parseInt(accountInfo.result.value.sequence);
    const initTxs = arr.map((_, index) => {
        console.log("Process:", index);
        return handleMsgSend({
            client,
            clientAddress: derivedAddress,
            sequence: sequence + index,
            toAddress: addresses.map((item) => item[0])[index],
        });
    });
    await Promise.all(initTxs);
    await (0, promises_1.setTimeout)(5000);
    console.log("Starting broadcasting transaction after 5 seconds...");
    //   console.time("Start timing...");
    const startTime = new Date().getTime();
    shouldPrintError = true;
    const executeTxs = arr.map((_, index) => {
        console.log("Process:", index);
        return handleExecutePerAccount(addresses.map((item) => item[1])[index], numberOfTxsPerAccount);
    });
    await Promise.all(executeTxs);
    const endTime = new Date().getTime();
    console.log("Total took:", (endTime - startTime) / 1000);
    console.timeEnd("End timing...");
}
main().then(() => {
    process.exit(0);
}, (error) => {
    console.error(error);
    process.exit(1);
});
