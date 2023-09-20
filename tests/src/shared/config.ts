export type Config = {
  bech32AddressPrefix: string;
  rpcUrl: string;
  gasPrice: number;
  feeDenom: string;
  mnemonic: string;
};

export const fetchConfig = async (): Promise<Config> => {
  if (process.env.BECH32_ADDRESS_PREFIX === undefined) {
    throw new Error('Missing BECH32_ADDRESS_PREFIX environment variable');
  }
  if (process.env.NET_URL === undefined) {
    throw new Error('Missing NET_URL environment variable');
  }
  if (process.env.GAS_PRICE === undefined) {
    throw new Error('Missing GAS_PRICE environment variable');
  }
  if (process.env.FEE_DENOM === undefined) {
    throw new Error('Missing FEE_DENOM environment variable');
  }
  if (process.env.MNEMONIC === undefined) {
    throw new Error('Missing MNEMONIC environment variable');
  }

  return {
    bech32AddressPrefix: process.env.BECH32_ADDRESS_PREFIX,
    rpcUrl: process.env.NET_URL,
    feeDenom: process.env.FEE_DENOM,
    gasPrice: parseFloat(process.env.GAS_PRICE),
    mnemonic: process.env.MNEMONIC,
  };
};
