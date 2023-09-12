import { fetchConfig } from '../shared/config';
import { createAdminCosmWasmClient, execute, getWallet, uploadAndInstantiate } from '../shared/cosmwasm';
import { Coin, coin } from '@cosmjs/proto-signing';
import { createCosmWasmClientForWallet, createWallet } from './helpers';
import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { HttpBatchClient } from '@cosmjs/tendermint-rpc/build/rpcclients';
import { kujiraQueryClient } from 'kujira.js';
import { PositionType } from '../types/dca/response/get_vault';
import { Pair } from '../types/exchanges/fin/pair';

const dexSwapFee = 0.0165;
const automationFee = 0.0075;
const finTakerFee = 0.0015;
const finMakerFee = 0.00075;
const finBuyPrice = 1.01;
const finSellPrice = 0.99;
const swapAdjustment = 1.3;

export const mochaHooks = async (): Promise<Mocha.RootHookObject> => {
  if (process.env.BECH32_ADDRESS_PREFIX !== 'kujira') {
    return;
  }

  const config = await fetchConfig();
  const httpClient = new HttpBatchClient(config.netUrl, {
    dispatchInterval: 100,
    batchSizeLimit: 200,
  });
  const tmClient = (await Tendermint34Client.create(httpClient)) as any;
  const queryClient = kujiraQueryClient({ client: tmClient });

  const cosmWasmClient = await createAdminCosmWasmClient(config);

  const adminWalletAddress = (
    await (await getWallet(config.adminWalletMnemonic, config.bech32AddressPrefix)).getAccounts()
  )[0].address;

  const feeCollectorWallet = await createWallet(config);
  const feeCollectorAddress = (await feeCollectorWallet.getAccounts())[0].address;

  const finPairAddress = await instantiateFinPairContract(cosmWasmClient, adminWalletAddress);

  const pairConfig = {
    ...(await cosmWasmClient.queryContractSmart(finPairAddress, {
      config: {},
    })),
    address: finPairAddress,
  };

  const dcaContractAddress = await instantiateDCAContract(cosmWasmClient, adminWalletAddress, feeCollectorAddress);

  const exchangeContractAddress = await instantiateExchangeContract(
    cosmWasmClient,
    adminWalletAddress,
    dcaContractAddress,
  );

  await migrateDCAContract(cosmWasmClient, adminWalletAddress, dcaContractAddress, exchangeContractAddress);

  const pair: Pair = {
    base_denom: pairConfig.denoms[1].native,
    quote_denom: pairConfig.denoms[0].native,
    address: finPairAddress,
    decimal_delta: 0,
    price_precision: 3,
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

  await cosmWasmClient.sendTokens(
    adminWalletAddress,
    userWalletAddress,
    [coin('100000000', pair.base_denom), coin('100000000', pair.quote_denom)],
    2,
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
        exchangeContractAddress,
        dexSwapFee,
        automationFee,
        adminWalletAddress,
        feeCollectorAddress,
        userWalletAddress,
        finPairAddress,
        finBuyPrice,
        finSellPrice,
        finMakerFee,
        finTakerFee,
        pair: {
          denoms: [pair.base_denom, pair.quote_denom],
        },
        validatorAddress,
        swapAdjustment,
      };

      Object.assign(this, context);
    },
  };
};

const instantiateDCAContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminWalletAddress: string,
  feeCollectorAdress: string,
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
      twap_period: 0,
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
  await execute(cosmWasmClient, adminWalletAddress, dcaContractAddress, {
    update_config: {
      exchange_contract_address: exchangeContractAddress,
    },
  });
};

export const instantiateExchangeContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminWalletAddress: string,
  dcaContractAddress: string,
): Promise<string> =>
  await uploadAndInstantiate(
    '../artifacts/fin.wasm',
    cosmWasmClient,
    adminWalletAddress,
    {
      admin: adminWalletAddress,
      dca_contract_address: dcaContractAddress,
    },
    'fin exchange',
  );

export const instantiateFinPairContract = async (
  cosmWasmClient: SigningCosmWasmClient,
  adminWalletAddress: string,
  baseDenom: string = 'utest',
  quoteDenom: string = 'udemo',
  beliefPrice: number = 1.0,
  orders: Record<string, number | Coin>[] = [],
): Promise<string> => {
  const finContractAddress = await uploadAndInstantiate(
    './src/artifacts/fin.wasm',
    cosmWasmClient,
    adminWalletAddress,
    {
      owner: adminWalletAddress,
      denoms: [{ native: baseDenom }, { native: quoteDenom }],
      price_precision: { decimal_places: 3 },
    },
    'fin',
  );

  await execute(cosmWasmClient, adminWalletAddress, finContractAddress, {
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
      adminWalletAddress,
      finContractAddress,
      {
        submit_order: { price: `${order.price}` },
      },
      [order.amount as Coin],
    );
  }

  return finContractAddress;
};
