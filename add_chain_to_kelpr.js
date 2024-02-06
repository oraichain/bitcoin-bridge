window.keplr.experimentalSuggestChain({
    rpc: 'http://127.0.0.1:26657',
    rest: 'http://127.0.0.1:8000',
    chainId: 'oraibtc-testnet-1',
    chainName: 'OraiBtc Local Testnet',
    bech32Config: {
        bech32PrefixAccAddr: 'oraibtc',
        bech32PrefixAccPub: 'oraibtc' + 'pub',
        bech32PrefixValAddr: 'oraibtc' + 'valoper',
        bech32PrefixValPub: 'oraibtc' + 'valoperpub',
        bech32PrefixConsAddr: 'oraibtc' + 'valcons',
        bech32PrefixConsPub: 'oraibtc' + 'valconspub',
    },
    stakeCurrency: {
        coinDenom: 'ORAIBTC',
        coinMinimalDenom: 'uoraibtc',
        coinDecimals: 6,
        gasPriceStep: {
            low: 0,
            average: 0,
            high: 0
        },
        coinImageUrl: 'https://assets.coingecko.com/coins/images/1/small/bitcoin.png'
    },
    bip44: {
        coinType: 118
    },
    coinType: 118,
    currencies: [
        {
            coinDenom: 'ORAIBTC',
            coinMinimalDenom: 'uoraibtc',
            coinDecimals: 6,
            gasPriceStep: {
                low: 0,
                average: 0,
                high: 0
            },
            coinImageUrl: 'https://assets.coingecko.com/coins/images/1/small/bitcoin.png'
        },
        {
            coinDenom: 'oBTC',
            coinMinimalDenom: 'usat',
            coinDecimals: 14,
            gasPriceStep: {
                low: 0,
                average: 0,
                high: 0
            },
            coinImageUrl: 'https://assets.coingecko.com/coins/images/1/small/bitcoin.png'
        }
    ],

    get feeCurrencies() {
        return this.currencies;
    }
});