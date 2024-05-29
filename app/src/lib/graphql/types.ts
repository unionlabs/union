import type { introspection } from "$generated/graphql-env"

type HasFields<T> = T extends { fields: any } ? T["fields"] : never

export type GraphQLTypes<T extends keyof introspection["types"]> = HasFields<
  introspection["types"][T]
>
