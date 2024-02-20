export const prerender = true

import { json } from '@sveltejs/kit'
import type { RequestHandler } from '@sveltejs/kit'
import { getUnoFromFaucet } from '$/lib/fetchers/faucet.ts'

export const GET: RequestHandler = async request => {
  const address = request.url.searchParams.get('address')

  try {
    if (!address) throw new Error('address is required')

    const response = await getUnoFromFaucet({ address })
    if (!Object.hasOwn(response, 'union')) {
      throw new Error(`Failed to get UNO from faucet: ${JSON.stringify(response, undefined, 2)}`)
    }

    return json(null, { status: 200 })
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : error
    console.error(errorMessage)
    return json({ error: errorMessage }, { status: 500 })
  }
}
