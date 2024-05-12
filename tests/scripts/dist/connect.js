"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
exports.connect = void 0;
const cosmwasm_stargate_1 = require("@cosmjs/cosmwasm-stargate");
const stargate_1 = require("@cosmjs/stargate");
const proto_signing_1 = require("@cosmjs/proto-signing");
const amino_1 = require("@cosmjs/amino");
const stargate_2 = require("@cosmjs/stargate");
/**
 *
 * @param mnemonic
 * @param network
 * @returns
 **/
async function connect(mnemonic, network, offline = true) {
    try {
        const { prefix, gasPrice, rpcEndpoint } = network;
        const hdPath = (0, amino_1.makeCosmoshubPath)(0);
        // Setup signer
        let signer;
        let address;
        if (offline) {
            const offlineSigner = await proto_signing_1.DirectSecp256k1HdWallet.fromMnemonic(mnemonic, {
                prefix,
                hdPaths: [hdPath],
            });
            const { address: addr } = (await offlineSigner.getAccounts())[0];
            signer = offlineSigner;
            address = addr;
        }
        else {
            const onlineSigner = await amino_1.Secp256k1HdWallet.fromMnemonic(mnemonic);
            const { address: addr } = (await onlineSigner.getAccounts())[0];
            signer = onlineSigner;
            address = addr;
        }
        const customAminoConverter = {
            "nomic/MsgSetRecoveryAddress": {
                aminoType: "nomic/MsgSetRecoveryAddress",
                toAmino: ({ recovery_address }) => ({
                    recovery_address: recovery_address,
                }),
                fromAmino: ({ recovery_address }) => ({
                    recovery_address: recovery_address,
                }),
            },
        };
        // Init SigningCosmWasmClient client
        const client = await cosmwasm_stargate_1.SigningCosmWasmClient.connectWithSigner(rpcEndpoint, signer, {
            gasPrice,
            aminoTypes: new stargate_2.AminoTypes({
                ...(0, stargate_1.createDefaultAminoConverters)(),
                ...(0, cosmwasm_stargate_1.createWasmAminoConverters)(),
                ...customAminoConverter,
            }),
        });
        return { client, address };
    }
    catch (err) {
        return await connect(mnemonic, network, offline);
    }
}
exports.connect = connect;
// Benchmarking results:
