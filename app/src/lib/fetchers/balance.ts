import { fetcher } from '$/lib/utilities/index.ts'
import { CHAIN, UNO, URLS } from '$/lib/constants.ts'

export async function fetchUnionUnoBalance(address: string) {
  const constructedURL = `${URLS.UNION.REST}/cosmos/bank/v1beta1/balances/${address}/by_denom?denom=${UNO.NATIVE_DENOM}`
  const response = await fetcher<
    | { balance: { denom: string; amount: string } }
    | { code: number; message: string; details: Array<unknown> }
  >(constructedURL, { method: 'GET' })

  if ('code' in response) {
    console.error(response)
    return '0'
  }

  return response.balance.amount
}
