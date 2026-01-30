// Initialize runtime on client startup
// This ensures the Effect runtime is ready before any pages load
import { getCurrentChain, getCurrentRuntime } from "$lib/runtime"

// Initialize runtime (side effect ensures it's ready)
getCurrentChain()
getCurrentRuntime()
