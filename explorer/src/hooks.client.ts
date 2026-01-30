// Initialize runtime on client startup
// This ensures the Effect runtime is ready before any pages load
import { getCurrentRuntime, getCurrentChain } from "$lib/runtime"

// Log runtime initialization
const chain = getCurrentChain()
const runtime = getCurrentRuntime()
console.log(`[Union Explorer] Runtime initialized for ${chain.pretty_name}`)
console.log(`[Union Explorer] REST endpoint: ${runtime.config.restEndpoint}`)
