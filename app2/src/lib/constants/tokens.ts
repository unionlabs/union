import { TokenRawDenom } from "@unionlabs/sdk/schema"

/**
 * List of token denoms that should be excluded from display and processing
 */
export const TOKEN_BLACKLIST: Array<TokenRawDenom> = [
  TokenRawDenom.make("0x0000000000000000000000000000000000000000"),
  TokenRawDenom.make("0xb7fb16053a3e3d4306791045769ec686f6ec4432")
]

/**
 * Checks if a token denom is blacklisted
 */
export const isTokenBlacklisted = (denom: TokenRawDenom): boolean => {
  return TOKEN_BLACKLIST.includes(denom)
}
