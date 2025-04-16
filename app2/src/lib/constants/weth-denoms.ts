import { TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"

export const WETH_DENOMS: Record<UniversalChainId, TokenRawDenom> = {
  [UniversalChainId.make("ethereum.11155111")]: TokenRawDenom.make(
    "0x7b79995e5f793a07bc00c21412e50ecae098e7f9"
  ),
  [UniversalChainId.make("ethereum.17000")]: TokenRawDenom.make(
    "0x94373a4919b3240d86ea41593d5eba789fef3848"
  ),
  //Bob mainnet
  [UniversalChainId.make("bob.60808")]: TokenRawDenom.make(
    "0x4200000000000000000000000000000000000006"
  ),
  //Bob testnet
  [UniversalChainId.make("bob.808813")]: TokenRawDenom.make(
    "0x4200000000000000000000000000000000000006"
  ),
  //Corn testnet
  [UniversalChainId.make("corn.21000001")]: TokenRawDenom.make(
    "0xda5ddd7270381a7c2717ad10d1c0ecb19e3cdfb2"
  )
}
