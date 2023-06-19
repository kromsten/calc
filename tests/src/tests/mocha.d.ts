import { SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate';
import { Config } from '../shared/config';
import { Addr } from '../types/dca/execute';
import { Pair } from '../types/dca/response/get_pairs';
import * as mocha from 'mocha';
import { KujiraQueryClient } from 'kujira.js';

declare module 'mocha' {
  export interface Context {
    config: Config;
    cosmWasmClient: SigningCosmWasmClient;
    queryClient: KujiraQueryClient;
    userCosmWasmClient: SigningCosmWasmClient;
    dcaContractAddress: Addr;
    exchangeContractAddress: Addr;
    dexSwapFee: number;
    automationFee: number;
    adminWalletAddress: Addr;
    feeCollectorAddress: Addr;
    userWalletAddress: Addr;
    finPairAddress: Addr;
    finBuyPrice: number;
    finSellPrice: number;
    finMakerFee: number;
    finTakerFee: number;
    pair: Pair;
    validatorAddress: string;
    swapAdjustment: number;
  }
}
