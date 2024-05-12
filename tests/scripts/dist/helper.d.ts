export declare const getAccountInfo: (address: string) => Promise<any>;
export declare function deriveAddressByPrefix(addr: string, prefix: string): string;
export declare function getMnemonic(): string;
export declare function getBlock(blockNumber: number): Promise<any>;
