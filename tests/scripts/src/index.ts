import { deriveAddressByPrefix, getMnemonic } from "./helper";
import { connect } from "./connect";
import { OraiBtcLocalConfig } from "./network";
import { getAccountInfo } from "./helper";
import { Worker } from "worker_threads";
import { coin, DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";
import { setTimeout } from "timers/promises";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { StdFee } from "@cosmjs/stargate";
import { TxRaw } from "cosmjs-types/cosmos/tx/v1beta1/tx";

const handleMsgSend = async ({
  client,
  clientAddress,
  sequence,
  toAddress,
}: {
  client: SigningCosmWasmClient;
  clientAddress: string;
  sequence: number;
  toAddress: string;
}): Promise<any> => {
  try {
    const sendMsg1 = {
      typeUrl: "/cosmos.bank.v1beta1.MsgSend",
      value: {
        fromAddress: clientAddress,
        toAddress, // a sample of to address
        amount: [
          {
            denom: OraiBtcLocalConfig.feeToken,
            amount: "0",
          },
        ],
      },
    };

    const txRaw = await client.sign(
      clientAddress,
      [sendMsg1],
      {
        amount: [coin("0", OraiBtcLocalConfig.feeToken)],
        gas: "100000",
      } as StdFee,
      "",
      {
        accountNumber: 0,
        chainId: OraiBtcLocalConfig.chainId,
        sequence,
      }
    );
    const txBytes = TxRaw.encode(txRaw).finish();
    const txData = await client.broadcastTx(txBytes);
    return txData;
  } catch (err: any) {
    await setTimeout(100);
    return await handleMsgSend({ client, clientAddress, sequence, toAddress });
  }
};

let mappedAddress = {} as { [key: string]: number };

const genNewAccount = async (): Promise<[string, string]> => {
  try {
    const mnemonic = (await DirectSecp256k1HdWallet.generate()).mnemonic;
    if (mappedAddress[`${mnemonic}`] !== undefined) {
      throw Error("Generate again");
    }
    console.log(mnemonic);
    mappedAddress[`${mnemonic}`] = 1;
    const { address } = await connect(mnemonic, OraiBtcLocalConfig, true);
    const derivedAddress = deriveAddressByPrefix(
      address,
      OraiBtcLocalConfig.prefix
    );
    return [derivedAddress, mnemonic];
  } catch (err) {
    console.log("Gen again");
    await setTimeout(100);
    return await genNewAccount();
  }
};

const runService = (workerData: { mnemonic: string; numOfTxs: number }) => {
  return new Promise((resolve, reject) => {
    const worker = new Worker("./dist/worker.js", {
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

async function main(): Promise<void> {
  // get the mnemonic
  const mnemonic = getMnemonic();
  const args = process.argv.slice(2);
  if (!args[0]) throw Error("Please input number of threads");
  if (!args[1]) throw Error("Please input number of txs per thread");
  const numberOfThread = parseInt(args[0]);
  const numberOfTxs = parseInt(args[1]);
  if (numberOfThread > 30) {
    throw Error("Threads is too large, may lead to overloaded");
  }
  const { address, client } = await connect(mnemonic, OraiBtcLocalConfig, true);
  const derivedAddress = deriveAddressByPrefix(
    address,
    OraiBtcLocalConfig.prefix
  );

  const accountInfo = await getAccountInfo(derivedAddress);
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
  const result = await Promise.all(
    array.map((x, index) =>
      runService({
        mnemonic: addresses[index][1],
        numOfTxs: numberOfTxs,
      })
    )
  );

  const endTime = new Date().getTime();

  const waitSecondsTime = (endTime - startTime) / 1000;
  console.log("Total taked: ", waitSecondsTime);
  const total = result
    .map((item: any) => item.result)
    .reduce((accumulator: any, currentValue: any) => {
      return accumulator + currentValue;
    }, 0);
  console.log("Tps:", total / waitSecondsTime);
  console.log(
    "Total executed txs: ",
    result
      .map((item: any) => item.result)
      .reduce((accumulator: any, currentValue: any) => {
        return accumulator + currentValue;
      }, 0)
  ); // Giá trị ban đầu của accumulator là 0
}

main().then(
  () => {
    process.exit(0);
  },
  (error) => {
    console.error(error);
    process.exit(1);
  }
);
