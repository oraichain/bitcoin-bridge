"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.OraiBtcLocalConfig = exports.OraiBtcMainnetConfig = void 0;
// data from https://github.com/cosmos/chain-registry/tree/master/testnets
const stargate_1 = require("@cosmjs/stargate");
exports.OraiBtcMainnetConfig = {
    chainId: "oraibtc-mainnet-1",
    rpcEndpoint: "https://btc.rpc.orai.io",
    prefix: "oraibtc",
    gasPrice: stargate_1.GasPrice.fromString("0uoraibtc"),
    feeToken: "uoraibtc",
    faucetUrl: "",
};
exports.OraiBtcLocalConfig = {
    chainId: "oraibtc-testnet-1",
    rpcEndpoint: "http://127.0.0.1:26657",
    prefix: "oraibtc",
    gasPrice: stargate_1.GasPrice.fromString("0uoraibtc"),
    feeToken: "uoraibtc",
    faucetUrl: "",
};
