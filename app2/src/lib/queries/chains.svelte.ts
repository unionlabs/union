import { Chains } from "$lib/schemas/chain"
import { createQueryGraphql } from "$lib/utils/queries"
import { ParseResult, Schema } from "effect"
import { graphql } from "gql.tada"
import { chains } from "$lib/stores/chains.svelte"

const ChainsResponseSchema = Schema.Struct({ v1_ibc_union_chains: Chains })

const ChainsFromResponse = Schema.transformOrFail(ChainsResponseSchema, Chains, {
  strict: true,
  decode: input => ParseResult.succeed(input.v1_ibc_union_chains),
  encode: (x, _, ast) => ParseResult.fail(new ParseResult.Forbidden(ast, x, "I will never encode"))
})

export let chainsQuery = createQueryGraphql({
  schema: ChainsFromResponse,
  document: graphql(`
    query Chains {
      v1_ibc_union_chains(where: {enabled: {_eq: true}}) {
        chain_id,
        display_name,
        addr_prefix
      }
    }
  `),
  refetchInterval: "5 seconds",
  writeData: data => {
    chains.data = data
  },
  writeError: error => {
    chains.error = error
  }
})
