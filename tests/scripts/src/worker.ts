import { deriveAddressByPrefix, getMnemonic } from "./helper";
import { connect } from "./connect";
import { OraiBtcLocalConfig } from "./network";
import { coin, StdFee } from "@cosmjs/amino";
// import { MsgSend } from "cosmjs-types/cosmos/bank/v1beta1/tx";
import { TxRaw } from "cosmjs-types/cosmos/tx/v1beta1/tx";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { setTimeout } from "timers/promises";
import { workerData, parentPort } from "worker_threads";

const toAddressConstant = "oraibtc1ehmhqcn8erf3dgavrca69zgp4rtxj5kqzpga4j";

const { mnemonic, numOfTxs } = workerData as {
  mnemonic: string;
  numOfTxs: number;
};

const getAccount = async (mnemonic: string): Promise<[any, any]> => {
  try {
    const { client, address } = await connect(
      mnemonic,
      OraiBtcLocalConfig,
      true
    );
    const derivedAddress = deriveAddressByPrefix(
      address,
      OraiBtcLocalConfig.prefix
    );
    return [client, derivedAddress];
  } catch (err) {
    return await getAccount(mnemonic);
  }
};

const handleMsgSend = async ({
  client,
  clientAddress,
  sequence,
  toAddress,
  count = 0,
}: {
  client: SigningCosmWasmClient;
  clientAddress: string;
  sequence: number;
  toAddress: string;
  count?: number;
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
    return 1;
  } catch (err: any) {
    if (count > 3) {
      return undefined;
    }
    await setTimeout(1000);
    return await handleMsgSend({
      client,
      clientAddress,
      sequence,
      toAddress,
      count: count + 1,
    });
  }
};

async function executor(mnemonic: string, numOfTxs: number): Promise<number> {
  // get the mnemonic
  const [client, derivedAddress] = await getAccount(mnemonic);
  const array = new Array(numOfTxs).fill(0);
  const data = await Promise.all(
    array.map((_, index) => {
      return handleMsgSend({
        client,
        clientAddress: derivedAddress,
        sequence: index,
        toAddress: toAddressConstant,
      });
    })
  );
  return data.filter((item) => item !== undefined).length;
}

(async () => {
  const result = await executor(mnemonic, numOfTxs);
  if (parentPort) {
    parentPort.postMessage({ result, status: "Done" });
  }
})();
