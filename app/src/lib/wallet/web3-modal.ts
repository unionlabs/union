/**
 * WIP: This file is not ready yet
 */
import { KEY } from "$lib/constants/keys.ts"
import { createWeb3Modal } from "@web3modal/wagmi"
import { config as evmConfig } from "./evm/config.ts"

createWeb3Modal({
  themeMode: "dark",
  wagmiConfig: evmConfig,
  projectId: KEY.WALLET_CONNECT_PROJECT_ID,
  enableOnramp: false,
  enableAnalytics: false,
  enableWalletFeatures: true,
  featuredWalletIds: [
    "1ae92b26df02f0abca6304df07debccd18262fdf5fe82daa81593582dac9a369", // rainbow
    "c57ca95b47569778a828d19178114f4db188b89b763c899ba0be274e97267d96", // metamask
    "ecc4036f814562b41a5268adc86270fba1365471402006302e70169465b7ac18", // zerion
    "18388be9ac2d02726dbac9777c96efaac06d744b2f6d580fccdd4127a6d01fd1" // rabby
  ],
  includeWalletIds: [
    "1ae92b26df02f0abca6304df07debccd18262fdf5fe82daa81593582dac9a369", // rainbow
    "c57ca95b47569778a828d19178114f4db188b89b763c899ba0be274e97267d96", // metamask
    "ecc4036f814562b41a5268adc86270fba1365471402006302e70169465b7ac18", // zerion
    "18388be9ac2d02726dbac9777c96efaac06d744b2f6d580fccdd4127a6d01fd1" // rabby
  ]
})
