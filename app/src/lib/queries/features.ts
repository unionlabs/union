import { URLS } from "$lib/constants"
import request from "graphql-request"
import { enabledFeatures } from "$lib/graphql/queries/features.ts"

export async function fetchFeatures(environment: string) {
  const { v1_ibc_union_chains } = await request(URLS().GRAPHQL, enabledFeatures, { environment })
  return v1_ibc_union_chains
}
