import SafeAppsSDK, { type Opts } from "@safe-global/safe-apps-sdk"

const opts: Opts = {
  allowedDomains: [
    /gnosis-safe.io$/,
    /app.safe.global$/,
    /staging.btc.union.build$/,
    /staging.app.union.build$/,
    /btc.union.build$/,
  ],
  debug: false,
}

export const safeOpts = opts
export const safeWallet = new SafeAppsSDK(opts)
