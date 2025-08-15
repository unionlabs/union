import { InternalChainInfo } from "./internal-chain-info";

export const xionTestnet = InternalChainInfo.make(
    {
      rpc: "https://rpc.xion-testnet-2.xion.chain.kitchen",
      rest: "https://rest.xion-testnet-2.xion.chain.kitchen",
      chainId: "xion-testnet-2",
      chainName: "Xion Testnet",
      chainSymbolImageUrl:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
      stakeCurrency: {
        coinDenom: "XION",
        coinMinimalDenom: "uxion",
        coinDecimals: 6,
        coinImageUrl:
          "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
      },
      bip44: {
        coinType: 118,
      },
      bech32Config: {
        bech32PrefixAccAddr: "xion",
        bech32PrefixAccPub: "xionpub",
        bech32PrefixValAddr: "xionvaloper",
        bech32PrefixValPub: "xionvaloperpub",
        bech32PrefixConsAddr: "xionvalcons",
        bech32PrefixConsPub: "xionvalconspub",
      },
      currencies: [
        {
          coinDenom: "XION",
          coinMinimalDenom: "uxion",
          coinDecimals: 6,
          coinImageUrl:
            "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
        },
      ],
      feeCurrencies: [
        {
          coinDenom: "XION",
          coinMinimalDenom: "uxion",
          coinDecimals: 6,
          coinImageUrl:
            "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
          gasPriceStep: {
            low: 0.001,
            average: 0.001,
            high: 0.002,
          },
        },
      ],
      features: [
        "cosmwasm",
      ],
      nodeProvider: {
        name: "🔥BurntLabs🔥",
        email: "security@burnt.com",
        website: "https://xion.burnt.com",
      },
      theme: {
        gradient: "linear-gradient(180deg, rgba(50, 129, 250, 0.32) 0%, rgba(50, 129, 250, 0) 100%)",
        primaryColor: "#3281fa",
      },
      image:
        "https://raw.githubusercontent.com/chainapsis/keplr-chain-registry/main/images/xion-testnet/chain.png",
    },
  )