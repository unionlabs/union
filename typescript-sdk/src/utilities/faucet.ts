import { fetcher } from "./index.ts";
import { UNION_GRAPHQL_API } from "#/constants";

export async function getUnoFromFaucet({ address }: { address: string }) {
  const response = await fetcher<
    | { data: { union: { send: null } } }
    | {
        errors: Array<{
          message: string;
          extensions: { path: string; code: string };
        }>;
      }
  >(UNION_GRAPHQL_API, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: /* GraphQL */ `
        mutation GetUno($address: Address!) {
          union {
            send(input: { toAddress: $address })
          }
        }
      `,
      variables: { address },
      operationName: "GetUno",
    }),
  });

  if ("errors" in response) {
    const [error] = response.errors;
    console.error(error);
    throw new Error(error?.message);
  }

  return response.data;
}
