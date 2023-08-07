export type Config = {
  dcaContractAddress: string;
  startAfter: string;
  netUrl: string;
  chain: string;
  xdefiPartnerKey: string;
};

export const fetchConfig = (): Config => {
  if (process.env.DCA_CONTRACT_ADDRESS === undefined) {
    throw new Error('Missing DCA_CONTRACT_ADDRESS environment variable');
  }
  if (process.env.START_AFTER === undefined) {
    throw new Error('Missing START_AFTER environment variable');
  }
  if (process.env.NET_URL === undefined) {
    throw new Error('Missing NET_URL environment variable');
  }
  if (process.env.CHAIN === undefined) {
    throw new Error('Missing CHAIN environment variable');
  }
  if (process.env.XDEFI_PARTNER_ID === undefined) {
    throw new Error('Missing XDEFI_PARTNER_ID environment variable');
  }

  return {
    dcaContractAddress: process.env.DCA_CONTRACT_ADDRESS!,
    startAfter: process.env.START_AFTER!,
    netUrl: process.env.NET_URL!,
    chain: process.env.CHAIN!,
    xdefiPartnerKey: process.env.XDEFI_PARTNER_ID!,
  };
};
