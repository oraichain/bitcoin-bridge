import { Network } from "./network";
/**
 *
 * @param mnemonic
 * @param network
 * @returns
 **/
export declare function connect(mnemonic: string, network: Network, offline?: boolean): Promise<{
    client: any;
    address: string;
}>;
