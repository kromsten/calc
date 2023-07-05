import { fetchConfig } from '../shared/config';
import {
  createAdminCosmWasmClient,
  execute,
  getWallet,
  uploadAndInstantiate,
  uploadAndMigrate,
} from '../shared/cosmwasm';
import { createCosmWasmClientForWallet, createWallet } from './helpers';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { cosmos, osmosis } from 'osmojs';
import { find, map } from 'ramda';
import { PositionType } from '../types/dca/execute';
import { Pool } from 'osmojs/types/codegen/osmosis/gamm/pool-models/balancer/balancerPool';
import { coin } from '@cosmjs/proto-signing';
import { Pair } from '../types/exchanges/osmosis/pair';

const dexSwapFee = 0.0005;
const automationFee = 0.0075;
const swapAdjustment = 1.3;

export const mochaHooks = async (): Promise<Mocha.RootHookObject> => {
  if (process.env.BECH32_ADDRESS_PREFIX !== 'osmo') {
    return;
  }

  const config = await fetchConfig();

  const queryClient = await osmosis.ClientFactory.createRPCQueryClient({ rpcEndpoint: config.netUrl });
  const cosmWasmClient = await createAdminCosmWasmClient(config);

  const adminWalletAddress = (
    await (await getWallet(config.adminWalletMnemonic, config.bech32AddressPrefix)).getAccounts()
  )[0].address;

  const feeCollectorWallet = await createWallet(config);
  const feeCollectorAddress = (await feeCollectorWallet.getAccounts())[0].address;

  const twapPeriod = 60;

  const dcaContractAddress = await instantiateDCAContract(
    cosmWasmClient,
    adminWalletAddress,
    feeCollectorAddress,
    twapPeriod,
  );

  const exchangeContractAddress = await instantiateExchangeContract(cosmWasmClient, adminWalletAddress);

  await migrateDCAContract(cosmWasmClient, adminWalletAddress, dcaContractAddress, exchangeContractAddress);

  const denoms = ['stake', 'uion'];

  const pools = map(
    (pool: any) => osmosis.gamm.v1beta1.Pool.decode(pool.value) as Pool,
    (await queryClient.osmosis.gamm.v1beta1.pools({})).pools,
  );

  const pool = find((pool: Pool) => {
    const assets = map((asset) => asset.token.denom, pool.poolAssets);
    return assets.length == 2 && assets.includes(denoms[0]) && assets.includes(denoms[1]);
  }, pools);

  const pair: Pair = {
    base_denom: denoms[0],
    quote_denom: denoms[1],
    route: [Number(pool.id)],
  };

  await execute(cosmWasmClient, adminWalletAddress, exchangeContractAddress, {
    internal_msg: {
      msg: Buffer.from(
        JSON.stringify({
          create_pairs: {
            pairs: [pair],
          },
        }),
      ).toString('base64'),
    },
  });

  const userWallet = await createWallet(config);
  const userWalletAddress = (await userWallet.getAccounts())[0].address;
  const userCosmWasmClient = await createCosmWasmClientForWallet(
    config,
    cosmWasmClient,
    adminWalletAddress,
    userWallet,
  );

  await cosmWasmClient.sendTokens(adminWalletAddress, userWalletAddress, [coin('1000000000', config.feeDenom)], 2);

  const validatorAddress = (
    await queryClient.cosmos.staking.v1beta1.validators({
      status: cosmos.staking.v1beta1.bondStatusToJSON(cosmos.staking.v1beta1.BondStatus.BOND_STATUS_BONDED),
    })
  ).validators[0].operatorAddress;

  return {
    beforeAll(this: Mocha.Context) {
      const context = {
        config,
        cosmWasmClient,
        userCosmWasmClient,
        queryClient,
        dcaContractAddress,
        exchangeContractAddress,
        dexSwapFee: 0.0005,
        automationFee: 0.0075,
        adminWalletAddress,
        feeCollectorAddress,
        userWalletAddress,
        pair: {
          denoms: [pair.base_denom, pair.quote_denom],
        },
        validatorAddress,
        swapAdjustment,
        twapPeriod,
      };

      Object.assign(this, context);
    },
  };
};

export const instantiateExchangeContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminWalletAddress: string,
): Promise<string> =>
  await uploadAndInstantiate(
    '../artifacts/osmosis.wasm',
    cosmWasmClient,
    adminWalletAddress,
    {
      admin: adminWalletAddress,
    },
    'osmosis exchange',
  );

const instantiateDCAContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminWalletAddress: string,
  feeCollectorAdress: string,
  twapPeriod: number,
): Promise<string> => {
  const dcaContractAddress = await uploadAndInstantiate(
    '../artifacts/dca.wasm',
    cosmWasmClient,
    adminWalletAddress,
    {
      admin: adminWalletAddress,
      executors: [adminWalletAddress],
      automation_fee_percent: `${automationFee}`,
      fee_collectors: [{ address: feeCollectorAdress, allocation: '1.0' }],
      default_page_limit: 30,
      paused: false,
      default_slippage_tolerance: '0.05',
      twap_period: twapPeriod,
      default_swap_fee_percent: `${dexSwapFee}`,
      weighted_scale_swap_fee_percent: '0.01',
      risk_weighted_average_escrow_level: '0.05',
      old_staking_router_address: adminWalletAddress,
    },
    'dca',
  );

  for (const position_type of ['enter', 'exit']) {
    await execute(cosmWasmClient, adminWalletAddress, dcaContractAddress, {
      update_swap_adjustment: {
        strategy: {
          risk_weighted_average: {
            model_id: 30,
            base_denom: 'bitcoin',
            position_type: position_type as PositionType,
          },
        },
        value: `${swapAdjustment}`,
      },
    });
  }

  return dcaContractAddress;
};

export const migrateDCAContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminWalletAddress: string,
  dcaContractAddress: string,
  exchangeContractAddress: string,
) => {
  let configResponse = await cosmWasmClient.queryContractSmart(dcaContractAddress, {
    get_config: {},
  });

  await uploadAndMigrate('../artifacts/dca.wasm', cosmWasmClient, adminWalletAddress, dcaContractAddress, {
    ...configResponse.config,
    exchange_contract_address: exchangeContractAddress,
  });
};
