import { Schema } from "effect"
import { Hex } from "$lib/schema/hex"
import {AddressEvmCanonical} from "$lib/schema/address";

export const TokenRawDenom = Hex.pipe(Schema.brand("TokenRawDenom"))
export const TokenRawAmount = Schema.BigInt.pipe(Schema.brand("TokenRawAmount"))
export const EVMWethToken = AddressEvmCanonical.pipe(
  Schema.annotations({ message: () => "WETH token must be a valid EVM canonical address (e.g., 0x followed by 40 hex chars)" })
);
