"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const helper_1 = require("./helper");
const main = async () => {
    for (let i = 1587362; i < 1591355; i++) {
        const data = await (0, helper_1.getBlock)(i);
        const result = data.result;
        const txs = result.block.data.txs;
        if (txs.length > 0) {
            console.log(result.block.header.last_block_id.hash, txs.length, i);
        }
    }
};
main();
