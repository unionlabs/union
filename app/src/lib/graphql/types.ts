import type { TadaDocumentNode } from "gql.tada"
import type { introspection } from "$generated/graphql-env"

type HasFields<T> = T extends { fields: any } ? T["fields"] : never

export type GraphQLTypes<T extends keyof introspection["__schema"]["types"]> = HasFields<
  introspection["__schema"]["types"][number]
>

export type ExtractData<T> = T extends TadaDocumentNode<infer U, any, any> ? U : never
