#!/bin/sh

CHAINID=${CHAINID:-testing}
DENOM=${DENOM:-aarch}
BLOCK_GAS_LIMIT=${GAS_LIMIT:-75000000}

IAVL_CACHE_SIZE=${IAVL_CACHE_SIZE:-1562500}
QUERY_GAS_LIMIT=${QUERY_GAS_LIMIT:-5000000}
SIMULATION_GAS_LIMIT=${SIMULATION_GAS_LIMIT:-50000000}
MEMORY_CACHE_SIZE=${MEMORY_CACHE_SIZE:-1000}

# Build genesis file incl account for each address passed in
coins="10000000000000000000000$DENOM"
archwayd init --chain-id $CHAINID $CHAINID
archwayd keys add validator --keyring-backend="test"
archwayd genesis add-genesis-account validator $coins --keyring-backend="test"

# create account for each passed in address
for addr in "$@"; do
  echo "creating genesis account: $addr"
  archwayd genesis add-genesis-account $addr $coins --keyring-backend="test"
done

archwayd genesis gentx validator 10000000000000000000000$DENOM --chain-id $CHAINID --keyring-backend="test"
archwayd genesis collect-gentxs


cat ~/.archway/config/config.toml


# Set proper defaults and change ports
sed -i 's/"leveldb"/"goleveldb"/g' ~/.archway/config/config.toml
sed -i 's#"tcp://127.0.0.1:26657"#"tcp://0.0.0.0:26657"#g' ~/.archway/config/config.toml
sed -i "s/\"stake\"/\"$DENOM\"/g" ~/.archway/config/genesis.json
sed -i "s/\"max_gas\": \"-1\"/\"max_gas\": \"$BLOCK_GAS_LIMIT\"/" ~/.archway/config/genesis.json
sed -i 's/timeout_commit = "5s"/timeout_commit = "1s"/g' ~/.archway/config/config.toml
sed -i 's/timeout_propose = "3s"/timeout_propose = "1s"/g' ~/.archway/config/config.toml
sed -i 's/index_all_keys = false/index_all_keys = true/g' ~/.archway/config/config.toml

#sed -i "s/minimum-gas-prices = 781250/iavl-cache-size = $IAVL_CACHE_SIZE/g" ~/.archway/config/app.toml
sed -i "s/iavl-cache-size = 781250/iavl-cache-size = $IAVL_CACHE_SIZE/g" ~/.archway/config/app.toml
sed -i "s/query_gas_limit = 50000000/query_gas_limit = $QUERY_GAS_LIMIT/g" ~/.archway/config/app.toml
sed -i "s/simulation_gas_limit = 25000000/simulation_gas_limit = $SIMULATION_GAS_LIMIT/g" ~/.archway/config/app.toml
sed -i "s/memory_cache_size = 512/memory_cache_size = $MEMORY_CACHE_SIZE/g" ~/.archway/config/app.toml

# Start the stake
archwayd start --pruning=nothing --minimum-gas-prices 1aarch