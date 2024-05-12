"use strict";
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.getBlock = exports.getMnemonic = exports.deriveAddressByPrefix = exports.getAccountInfo = void 0;
const dotenv_1 = __importDefault(require("dotenv"));
dotenv_1.default.config();
const axios_1 = __importDefault(require("axios"));
const axios_extensions_1 = require("axios-extensions");
const oraidex_common_1 = require("@oraichain/oraidex-common");
const encoding_1 = require("@cosmjs/encoding");
const axios = axios_1.default.create({
    timeout: oraidex_common_1.AXIOS_TIMEOUT,
    retryTimes: 3,
    // cache will be enabled by default in 2 seconds
    adapter: (0, axios_extensions_1.retryAdapterEnhancer)((0, axios_extensions_1.throttleAdapterEnhancer)(axios_1.default.defaults.adapter, {
        threshold: oraidex_common_1.AXIOS_THROTTLE_THRESHOLD,
    })),
    baseURL: "http://127.0.0.1:8000",
});
const getAccountInfo = async (address) => {
    const res = await axios.get(`/auth/accounts/${address}`, {});
    return res.data;
};
exports.getAccountInfo = getAccountInfo;
function deriveAddressByPrefix(addr, prefix) {
    let address = (0, encoding_1.fromBech32)(addr);
    return (0, encoding_1.toBech32)(prefix, address.data);
}
exports.deriveAddressByPrefix = deriveAddressByPrefix;
// Check "MNEMONIC" env variable and ensure it is set to a reasonable value
function getMnemonic() {
    const mnemonic = process.env["MNEMONIC"];
    if (!mnemonic || mnemonic.length < 48) {
        throw new Error("Must set MNEMONIC to a 12 word phrase");
    }
    return mnemonic;
}
exports.getMnemonic = getMnemonic;
async function getBlock(blockNumber) {
    const axios = axios_1.default.create({
        timeout: oraidex_common_1.AXIOS_TIMEOUT,
        retryTimes: 3,
        // cache will be enabled by default in 2 seconds
        adapter: (0, axios_extensions_1.retryAdapterEnhancer)((0, axios_extensions_1.throttleAdapterEnhancer)(axios_1.default.defaults.adapter, {
            threshold: oraidex_common_1.AXIOS_THROTTLE_THRESHOLD,
        })),
        baseURL: "http://btc.rpc.orai.io",
    });
    const res = await axios.get("/block", {
        params: {
            height: blockNumber,
        },
    });
    return res.data;
}
exports.getBlock = getBlock;
