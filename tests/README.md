# **Calculated Finance Integration Tests**

## Running the tests

Each chain requires some slightly different setup. The following sections describe how to run the tests locally for each chain.

### Pre-requisites

#### Kujira

None

#### Osmosis

1. Run `git clone https://github.com/osmosis-labs/osmosis.git` from the tests directory.
2. Replace `uosmo` for `stake` in the `osmosis/tests/localosmosis/scripts/nativeDenomPoolB.json` file.
3. Replace `$STATE` fro `-s` in the `osmosis/tests/localosmosis/docker-compose.yml` file.

#### Duality

1. Run `git clone https://github.com/duality-labs/duality.git` from the tests directory.
2. `cd duality` && `git checkout devnet`.
3. Replace the version of the duality docker image in heighliner.Dockerfile with the latest image version from ghcr.io/duality-labs/duality.
4. Set `export STARTUP_MODE="${MODE:-new}"` in the `startup.sh` script.

### Tests

Run the tests via:

1. `npm run localnet:{{chain}}`
2. Wait for the docker container to initialise (around 1-2 mins depending on the chain).
3. `npm run test:{{chain}}`
