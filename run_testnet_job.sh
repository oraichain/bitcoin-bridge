pm2 start "FUNDED_ADDRESS=oraibtc1rchnkdpsxzhquu63y6r4j4t57pnc9w8ea88hue FUNDED_ORAIBTC_AMOUNT=100000000000 nomic start --chain-id oraibtc-testnet-1"
pm2 start "nomic signer --chain-id oraibtc-testnet-1"
pm2 start "nomic relayer --rpc-port=18332 --rpc-user=satoshi --rpc-pass=nakamoto --chain-id oraibtc-testnet-1"
pm2 start "bitcoind -server -testnet -rpcuser=satoshi -rpcpassword=nakamoto -prune=5000 -minrelaytxfee=0.000005 -datadir=/media/lenovo/DATABOX/Developer/.bitcoin-testnet"
pm2 start "nomic grpc --chain-id oraibtc-testnet-1 -g 0.0.0.0 -- 9001"

# nomic declare NcBawcjuMVWB8CRI55IFMwWHlCQeOqFZ4/DZCHgAlk0= 10000000000 0.042 0.1 0.01 10000000000 "Foo Bar" "Foo Bar" JASDHKAJSD "Foo Bar"
# RUST_LOG=debug cargo test --verbose --test bitcoin --features=full,feat-ibc,testnet,faucet-test,devnet