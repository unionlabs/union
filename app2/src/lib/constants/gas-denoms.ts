import { TokenRawDenom, UniversalChainId } from "@unionlabs/sdk/schema";

export const GAS_DENOMS: Record<UniversalChainId, TokenRawDenom> = {
  [UniversalChainId.make("ethereum.11155111")]: TokenRawDenom.make(
    "0x0000000000000000000000000000000000000000"
  ),
  [UniversalChainId.make("ethereum.1")]: TokenRawDenom.make(
    "0x0000000000000000000000000000000000000000"
  ),
  [UniversalChainId.make("ethereum.17000")]: TokenRawDenom.make(
    "0xeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee"
  ),
  [UniversalChainId.make("babylon.bbn-1")]: TokenRawDenom.make("0x7562626e"),
  [UniversalChainId.make("babylon.bbn-test-5")]:
    TokenRawDenom.make("0x7562626e"),
};
