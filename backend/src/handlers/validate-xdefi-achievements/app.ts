import dayjs from 'dayjs';
import relativeTime from 'dayjs/plugin/relativeTime';
import axios from 'axios';
import { fetchConfig } from '../../shared/config';
import { QueryClient } from '@cosmjs/stargate';
import { HttpBatchClient, Tendermint34Client } from '@cosmjs/tendermint-rpc';
import { setupWasmExtension } from '@cosmjs/cosmwasm-stargate';
dayjs.extend(relativeTime);

const STABLECOINS = {
  kujira: [
    'factory/kujira1qk00h5atutpsv900x202pxx42npjr9thg58dnqpa72f2p7m2luase444a7/uusk',
    'ibc/295548A78785A1007F232DE286149A6FF512F180AF5657780FC89C009E2C348F',
  ],
  osmosis: [
    'ibc/92BE0717F4678905E53F4E45B2DED18BC0CB97BF1F8B6A25AFEDF3D5A879B4D5',
    'ibc/8242AD24008032E457D2E12D46588FD39FB54FB29680C6C7663D296B383C37C4',
    'ibc/0CD3A0285E1341859B5E86B6AB7682F023D03E97607CCC1DC95706411D866DF7',
    'ibc/D189335C6E4A68B513C10AB227BF1C1D38C746766278BA3EEB4FB14124F1D858',
  ],
};

export const handler = async () => {
  const config = fetchConfig();

  const client = QueryClient.withExtensions(
    (await Tendermint34Client.create(new HttpBatchClient(config.netUrl, { dispatchInterval: 2000 }))) as any,
    setupWasmExtension,
  ).wasm;

  let startAfter = config.startAfter;

  while (true) {
    const result = await client.queryContractSmart(config.dcaContractAddress, {
      get_vaults: {
        limit: 300,
        start_after: startAfter,
      },
    });

    for (const vault of result.vaults) {
      const campaign = STABLECOINS[config.chain].includes(vault.deposited_amount.denom)
        ? 'calc_accumulate'
        : 'calc_takeprofit';

      console.log(`${vault.owner} has completed ${campaign} campaign`);

      const response = await axios.post(`https://campaign-ts.xdefi.services/api/campaigns/${campaign}/events`, {
        chain: config.chain,
        address: vault.owner,
        partnerName: 'calc',
        partnerKey: config.xdefiPartnerKey,
      });

      if (response.status !== 200) {
        console.error(`Failed to send event to xDefi for ${vault.owner}`);
      }
    }

    if (result.vaults.length < 100) {
      break;
    }

    startAfter += 100;
  }
};
