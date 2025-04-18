import SafeAppsSDK from '@safe-global/safe-apps-sdk';

type Opts = {
  allowedDomains?: RegExp[];
  debug?: boolean;
};

const opts: Opts = {
  allowedDomains: [
    /gnosis-safe.io$/,
    /app.safe.global$/,
    /staging.btc.union.build$/,
    /btc.union.build$/
  ],
  debug: false,
};

export const safeWallet = new SafeAppsSDK(opts);