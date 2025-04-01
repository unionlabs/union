import { TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema"

export const WETH_DENOMS: Record<UniversalChainId, TokenRawDenom> = {
  [UniversalChainId.make("ethereum.11155111")]: TokenRawDenom.make(
    "0x7b79995e5f793A07Bc00c21412e50Ecae098E7f9".toLowerCase() as `0x${string}`
  ),
  [UniversalChainId.make("ethereum.17000")]: TokenRawDenom.make(
    "0x94373a4919B3240D86eA41593D5eBa789FEF3848".toLowerCase() as `0x${string}`
  )
}
