import { GasPrice } from "@cosmjs/stargate";
export interface Network {
    chainId: string;
    rpcEndpoint: string;
    prefix: string;
    gasPrice: GasPrice;
    feeToken: string;
    faucetUrl: string;
}
export declare const OraiBtcMainnetConfig: Network;
export declare const OraiBtcLocalConfig: Network;
