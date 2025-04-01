/**
 * Simple REST function to get gas price from fee market
 */
export async function testFeeMarketResponse(restUrl: string, denom: string): Promise<any> {
  try {
    console.log(`Lukas: Querying gas price for denom: ${denom}...`)

    // Format the URL - ensure no trailing slash in base URL
    const baseUrl = restUrl.replace(/\/$/, "")
    const endpoint = `${baseUrl}/feemarket/v1/gas_price/${denom}`

    console.log(`Lukas: Fetching from ${endpoint}`)

    // Make a simple GET request
    const response = await fetch(endpoint)

    console.log(`Lukas: Response status: ${response.status}`)

    if (!response.ok) {
      const errorText = await response.text()
      console.error(`Lukas: Error: ${response.status}`, errorText)
      return {
        success: false,
        error: `HTTP error ${response.status}`,
        details: errorText
      }
    }

    // Parse the JSON response
    const data = await response.json()
    console.log(`Lukas: Response data:`, data)

    return {
      success: true,
      gasPrice: data
    }
  } catch (error) {
    console.error("Lukas: Error fetching gas price:", error)
    return {
      success: false,
      error: error instanceof Error ? error.message : String(error)
    }
  }
}
