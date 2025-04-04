import {Effect, Option} from "effect";
import {type AddressCanonicalBytes, Chain, Token} from "@unionlabs/sdk/schema";
import {balancesStore} from "$lib/stores/balances.svelte.ts";

export const checkBalance = (source: Chain, sender: AddressCanonicalBytes, token: Token, amount: string) => {
  return Effect.flatMap(
    Effect.sync(() => balancesStore.getBalance(source.universal_chain_id, sender, token.denom)),
    balance => {
      if (!Option.isSome(balance)) {
        console.log('[CTS] No balance found');
        return Effect.succeed(false);
      }

      return Effect.try({
        try: () => {
          const amountBigInt = BigInt(amount);
          const balanceBigInt = BigInt(balance.value);
          const enough = amountBigInt <= balanceBigInt;

          console.log('[CTS] Balance check:', {
            amount: amountBigInt.toString(),
            balance: balanceBigInt.toString(),
            enough
          });

          return enough;
        },
        catch: (error) => {
          console.error('[CTS] Error converting to BigInt:', error);
          return false;
        }
      });
    }
  );
};