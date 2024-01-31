pm2 start "FUNDED_ADDRESS=oraibtc1rchnkdpsxzhquu63y6r4j4t57pnc9w8ea88hue FUNDED_ORAIBTC_AMOUNT=100000000000 nomic start --chain-id oraibtc-subnet-1"
pm2 start "nomic signer --chain-id oraibtc-subnet-1"
pm2 start "nomic relayer --rpc-port=18332 --rpc-user=satoshi --rpc-pass=nakamoto --chain-id oraibtc-subnet-1"
pm2 start "bitcoind -server -testnet -rpcuser=satoshi -rpcpassword=nakamoto -prune=5000 -datadir=/media/lenovo/DATABOX/Developer/.bitcoin-testnet"