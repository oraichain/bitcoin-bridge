import dotenv from "dotenv";
dotenv.config();
import Axios from "axios";
import {
  throttleAdapterEnhancer,
  retryAdapterEnhancer,
} from "axios-extensions";
import {
  AXIOS_TIMEOUT,
  AXIOS_THROTTLE_THRESHOLD,
} from "@oraichain/oraidex-common";
import { fromBech32, toBech32 } from "@cosmjs/encoding";

const axios = Axios.create({
  timeout: AXIOS_TIMEOUT,
  retryTimes: 3,
  // cache will be enabled by default in 2 seconds
  adapter: retryAdapterEnhancer(
    throttleAdapterEnhancer(Axios.defaults.adapter!, {
      threshold: AXIOS_THROTTLE_THRESHOLD,
    })
  ),
  baseURL: "http://127.0.0.1:8000",
});

export const getAccountInfo = async (address: string): Promise<any> => {
  const res = await axios.get(`/auth/accounts/${address}`, {});
  return res.data;
};

export function deriveAddressByPrefix(addr: string, prefix: string) {
  let address = fromBech32(addr);
  return toBech32(prefix, address.data);
}

// Check "MNEMONIC" env variable and ensure it is set to a reasonable value
export function getMnemonic(): string {
  const mnemonic = process.env["MNEMONIC"];
  if (!mnemonic || mnemonic.length < 48) {
    throw new Error("Must set MNEMONIC to a 12 word phrase");
  }
  return mnemonic;
}
