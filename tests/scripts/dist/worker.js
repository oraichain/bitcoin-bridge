"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const helper_1 = require("./helper");
const connect_1 = require("./connect");
const network_1 = require("./network");
const amino_1 = require("@cosmjs/amino");
// import { MsgSend } from "cosmjs-types/cosmos/bank/v1beta1/tx";
const tx_1 = require("cosmjs-types/cosmos/tx/v1beta1/tx");
const promises_1 = require("timers/promises");
const worker_threads_1 = require("worker_threads");
const toAddressConstant = "oraibtc1ehmhqcn8erf3dgavrca69zgp4rtxj5kqzpga4j";
const { mnemonic, numOfTxs } = worker_threads_1.workerData;
const getAccount = async (mnemonic) => {
    try {
        const { client, address } = await (0, connect_1.connect)(mnemonic, network_1.OraiBtcLocalConfig, true);
        const derivedAddress = (0, helper_1.deriveAddressByPrefix)(address, network_1.OraiBtcLocalConfig.prefix);
        return [client, derivedAddress];
    }
    catch (err) {
        return await getAccount(mnemonic);
    }
};
const handleMsgSend = async ({ client, clientAddress, sequence, toAddress, count = 0, }) => {
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
        return 1;
    }
    catch (err) {
        if (count > 3) {
            return undefined;
        }
        await (0, promises_1.setTimeout)(1000);
        return await handleMsgSend({
            client,
            clientAddress,
            sequence,
            toAddress,
            count: count + 1,
        });
    }
};
async function executor(mnemonic, numOfTxs) {
    // get the mnemonic
    const [client, derivedAddress] = await getAccount(mnemonic);
    const array = new Array(numOfTxs).fill(0);
    const data = await Promise.all(array.map((_, index) => {
        return handleMsgSend({
            client,
            clientAddress: derivedAddress,
            sequence: index,
            toAddress: toAddressConstant,
        });
    }));
    return data.filter((item) => item !== undefined).length;
}
(async () => {
    const result = await executor(mnemonic, numOfTxs);
    if (worker_threads_1.parentPort) {
        worker_threads_1.parentPort.postMessage({ result, status: "Done" });
    }
})();
