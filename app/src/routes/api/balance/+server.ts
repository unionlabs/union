import { json } from '@sveltejs/kit'
import type { RequestHandler } from './$types.ts'
import { fetcher } from '$/lib/utilities/index.ts'
import { CHAIN, UNO, URLS } from '$/lib/constants.ts'

/**
 * Only supports UNION chain for now
 */

export const GET = (async request => {
  const chain = request.url.searchParams.get('chain')
  if (chain !== CHAIN.UNION.ID) return json({ error: 'chain not supported' }, { status: 400 })

  const address = request.url.searchParams.get('address')
  if (!address) return json({ error: 'address is required' }, { status: 400 })

  const constructedURL = `${URLS.UNION.REST}/cosmos/bank/v1beta1/balances/${address}/by_denom?denom=${UNO.NATIVE_DENOM}`
  const response = await fetcher<
    | { balance: { denom: string; amount: string } }
    | { code: number; message: string; details: Array<unknown> }
  >(constructedURL, { method: 'GET' })

  if ('code' in response) return json({ error: response.message }, { status: 400 })

  return new Response(response.balance.amount, { status: 200 })
}) satisfies RequestHandler
