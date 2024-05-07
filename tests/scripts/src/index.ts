import { deriveAddressByPrefix, getMnemonic } from "./helper";
import { connect } from "./connect";
import { OraiBtcLocalConfig } from "./network";
import { coin, StdFee } from "@cosmjs/amino";
// import { MsgSend } from "cosmjs-types/cosmos/bank/v1beta1/tx";
import { TxRaw } from "cosmjs-types/cosmos/tx/v1beta1/tx";
import { getAccountInfo } from "./helper";
import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";
import { setTimeout } from "timers/promises";
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing";

const toAddressConstant = "oraibtc1ehmhqcn8erf3dgavrca69zgp4rtxj5kqzpga4j";

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
}) => {
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
  } catch (err) {
    // console.log(err);
    await setTimeout(100);
    await handleMsgSend({ client, clientAddress, sequence, toAddress });
  }
};

const handleExecutePerAccount = async (
  mnemonic: string,
  numberOfTxs: number
) => {
  const { client, address } = await connect(mnemonic, OraiBtcLocalConfig, true);
  const derivedAddress = deriveAddressByPrefix(
    address,
    OraiBtcLocalConfig.prefix
  );

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

  //   console.log("Starting broadcasting transaction...");

  const data = await Promise.all(bulkTxs);

  //   console.log("Data", data);
};

const genNewAccount = async (): Promise<[string, string]> => {
  try {
    const mnemonic = (await DirectSecp256k1HdWallet.generate()).mnemonic;
    const { address } = await connect(mnemonic, OraiBtcLocalConfig, true);
    const derivedAddress = deriveAddressByPrefix(
      address,
      OraiBtcLocalConfig.prefix
    );
    return [derivedAddress, mnemonic];
  } catch (err) {
    return await genNewAccount();
  }
};

async function main(): Promise<void> {
  // get the mnemonic
  const mnemonic = getMnemonic();
  const args = process.argv.slice(2);
  if (!args[0]) throw Error("Please input number of accounts");
  if (!args[1]) throw Error("Please input number of txs per account");
  console.log(
    `Benchmarking with ${args[0]} accounts and ${args[1]} txs per each`
  );
  const numberOfAccount = parseInt(args[0]);
  const numberOfTxsPerAccount = parseInt(args[1]);

  const addressesFunc = new Array(numberOfAccount).fill(0).map((_, index) => {
    return genNewAccount();
  });
  const addresses = await Promise.all(addressesFunc);

  const { client, address } = await connect(mnemonic, OraiBtcLocalConfig, true);
  const derivedAddress = deriveAddressByPrefix(
    address,
    OraiBtcLocalConfig.prefix
  );
  const accountInfo = await getAccountInfo(derivedAddress);
  const sequence = parseInt(accountInfo.result.value.sequence);

  const initTxs = new Array(numberOfAccount).fill(0).map((_, index) => {
    return handleMsgSend({
      client,
      clientAddress: derivedAddress,
      sequence: sequence + index,
      toAddress: addresses.map((item) => item[0])[index],
    });
  });
  await Promise.all(initTxs);

  await setTimeout(5000);
  console.log("Starting broadcasting transaction after 5 seconds...");
  //   console.time("Start timing...");
  const startTime = new Date().getTime();
  const executeTxs = new Array(numberOfAccount).fill(0).map((_, index) => {
    return handleExecutePerAccount(
      addresses.map((item) => item[1])[index],
      numberOfTxsPerAccount
    );
  });
  await Promise.all(executeTxs);
  const endTime = new Date().getTime();
  console.log("Total took:", (endTime - startTime) / 1000);

  console.timeEnd("End timing...");
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
