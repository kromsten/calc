import dotenv from 'dotenv';
import { fetchConfig } from '../shared/config';
import { createAdminCosmWasmClient, execute, getWallet, uploadAndInstantiate } from '../shared/cosmwasm';
import { Coin, coin } from '@cosmjs/proto-signing';
import { createCosmWasmClientForWallet, createWallet } from './helpers';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { HttpBatchClient } from '@cosmjs/tendermint-rpc/build/rpcclients';
import { kujiraQueryClient } from 'kujira.js';
import { PositionType } from '../types/dca/response/get_vault';

const calcSwapFee = 0.0165;
const automationFee = 0.0075;
const finTakerFee = 0.0015;
const finMakerFee = 0.00075;
const finBuyPrice = 1.01;
const finSellPrice = 0.99;
const swapAdjustment = 1.3;

export const mochaHooks = async (): Promise<Mocha.RootHookObject> => {
  dotenv.config();

  const config = await fetchConfig();
  const httpClient = new HttpBatchClient(config.netUrl, {
    dispatchInterval: 100,
    batchSizeLimit: 200,
  });
  const tmClient = (await Tendermint34Client.create(httpClient)) as any;
  const queryClient = kujiraQueryClient({ client: tmClient });
  const cosmWasmClient = await createAdminCosmWasmClient(config);

  const adminContractAddress = (
    await (await getWallet(config.adminContractMnemonic, config.bech32AddressPrefix)).getAccounts()
  )[0].address;

  const feeCollectorWallet = await createWallet(config);
  const feeCollectorAddress = (await feeCollectorWallet.getAccounts())[0].address;

  const finPairAddress = await instantiateFinPairContract(cosmWasmClient, adminContractAddress);

  const pairConfig = {
    ...(await cosmWasmClient.queryContractSmart(finPairAddress, {
      config: {},
    })),
    address: finPairAddress,
  };

  const pair = {
    base_denom: pairConfig.denoms[0].native,
    quote_denom: pairConfig.denoms[1].native,
    address: finPairAddress,
  };

  const dcaContractAddress = await instantiateDCAContract(cosmWasmClient, adminContractAddress, feeCollectorAddress, [
    finPairAddress,
  ]);

  const userWallet = await createWallet(config);
  const userWalletAddress = (await userWallet.getAccounts())[0].address;
  const userCosmWasmClient = await createCosmWasmClientForWallet(
    config,
    cosmWasmClient,
    adminContractAddress,
    userWallet,
  );

  const validatorAddress = (await queryClient.staking.validators('')).validators[0].operatorAddress;

  return {
    beforeAll(this: Mocha.Context) {
      const context = {
        config,
        cosmWasmClient,
        queryClient,
        userCosmWasmClient,
        dcaContractAddress,
        calcSwapFee,
        automationFee,
        adminContractAddress,
        feeCollectorAddress,
        userWalletAddress,
        finPairAddress,
        finBuyPrice,
        finSellPrice,
        finMakerFee,
        finTakerFee,
        pair,
        validatorAddress,
        swapAdjustment,
      };

      Object.assign(this, context);
    },
  };
};

const instantiateDCAContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminContractAddress: string,
  feeCollectorAdress: string,
  pairAddress: string[] = [],
): Promise<string> => {
  const dcaContractAddress = await uploadAndInstantiate(
    '../artifacts/dca.wasm',
    cosmWasmClient,
    adminContractAddress,
    {
      admin: adminContractAddress,
      executors: [adminContractAddress],
      automation_fee_percent: `${automationFee}`,
      fee_collectors: [{ address: feeCollectorAdress, allocation: '1.0' }],
      default_page_limit: 30,
      paused: false,
      default_slippage_tolerance: '0.05',
      twap_period: 60,
      default_swap_fee_percent: `${calcSwapFee}`,
      weighted_scale_swap_fee_percent: '0.01',
      risk_weighted_average_escrow_level: '0.05',
    },
    'dca',
  );

  for (const address of pairAddress) {
    const pair = await cosmWasmClient.queryContractSmart(address, {
      config: {},
    });

    await execute(cosmWasmClient, adminContractAddress, dcaContractAddress, {
      create_pair: {
        base_denom: pair.denoms[0].native,
        quote_denom: pair.denoms[1].native,
        address,
      },
    });
  }

  for (const position_type of ['enter', 'exit']) {
    await execute(cosmWasmClient, adminContractAddress, dcaContractAddress, {
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

export const instantiateFinExchangeWrapperContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminContractAddress: string,
  pairAddress: string[] = [],
): Promise<string> => {
  const dcaContractAddress = await uploadAndInstantiate(
    '../artifacts/dca.wasm',
    cosmWasmClient,
    adminContractAddress,
    {},
    'dca',
  );

  for (const address of pairAddress) {
    const pair = await cosmWasmClient.queryContractSmart(address, {
      config: {},
    });

    await execute(cosmWasmClient, adminContractAddress, dcaContractAddress, {
      create_pair: {
        base_denom: pair.denoms[0].native,
        quote_denom: pair.denoms[1].native,
        address,
      },
    });
  }

  return dcaContractAddress;
};

export const instantiateFinPairContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminContractAddress: string,
  baseDenom: string = 'utest',
  quoteDenom: string = 'udemo',
  beliefPrice: number = 1.0,
  orders: Record<string, number | Coin>[] = [],
): Promise<string> => {
  const finContractAddress = await uploadAndInstantiate(
    './src/artifacts/fin.wasm',
    cosmWasmClient,
    adminContractAddress,
    {
      owner: adminContractAddress,
      denoms: [{ native: baseDenom }, { native: quoteDenom }],
      price_precision: { decimal_places: 3 },
    },
    'fin',
  );

  await execute(cosmWasmClient, adminContractAddress, finContractAddress, {
    launch: {},
  });

  orders =
    (orders.length == 0 && [
      { price: beliefPrice + 0.01, amount: coin('1000000000000', baseDenom) },
      { price: beliefPrice + 0.2, amount: coin('10000000000000', baseDenom) },
      { price: beliefPrice - 0.01, amount: coin('1000000000000', quoteDenom) },
      { price: beliefPrice - 0.2, amount: coin('10000000000000', quoteDenom) },
    ]) ||
    orders;

  for (const order of orders) {
    await execute(
      cosmWasmClient,
      adminContractAddress,
      finContractAddress,
      {
        submit_order: { price: `${order.price}` },
      },
      [order.amount as Coin],
    );
  }

  return finContractAddress;
};
