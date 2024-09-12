<script lang="ts">
import Navbar from "$lib/layout/Navbar/index.svelte"
import { beforeNavigate, goto } from "$app/navigation"
import { checkAuth, type SessionError } from "$lib/utils/auth.ts"
import { supabase } from "$lib/supabase/client.ts"
import { user } from "$lib/stores/user.svelte.ts"
import { Toaster } from "svelte-sonner"
import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query"
import { onMount } from "svelte"
import { Application } from "@splinetool/runtime"

import "../styles/tailwind.css"

let { children } = $props()

beforeNavigate(async ({ from, to, cancel }) => {
  const pathname = to?.route?.id
  if (pathname) {
    const segments = pathname.split("/").filter(Boolean)
    if (segments[0] === "app") {
      const authCheck = await checkAuth()

      authCheck.match(
        () => {
          console.log("User authenticated")
        },
        (error: SessionError) => {
          console.error(error.message)
          cancel()
          goto("/auth/register")
        }
      )
    }
  }
})

$effect(() => {
  const {
    data: { subscription }
  } = supabase.auth.onAuthStateChange((event, session) => {
    user.session = session
  })
  return () => {
    subscription.unsubscribe()
  }
})

let canvas: HTMLCanvasElement
let app: Application
let loading = $state(true)

onMount(() => {
  canvas = document.getElementById("canvas3d") as HTMLCanvasElement
  if (!canvas) {
    console.error("Canvas element not found")
    return
  }

  app = new Application(canvas)
  if (!app) {
    console.error("Failed to create Spline Application")
    return
  }

  app
    .load("https://prod.spline.design/6An57q5Kr37gF2k0/scene.splinecode")
    .then(splineScene => {
      loading = false
    })
    .catch(error => {
      console.error("Failed to load Spline scene:", error)
      loading = false
    })
})

const queryClient = new QueryClient()
</script>

<style>
    .canvas-fade {
        opacity: 0;
        transition: opacity 1s ease-in-out;
    }

    .canvas-fade.loaded {
        opacity: 1;
    }
</style>

<Toaster position="bottom-right" toastOptions={{ class: 'rounded-none border border-black',}}/>

<QueryClientProvider client={queryClient}>
  <Navbar/>
  <canvas
          id="canvas3d"
          class=" w-full h-auto inset-0 absolute -z-10 canvas-fade"
          class:loaded={!loading}
  ></canvas>

  <main class="flex flex-col items-center justify-center min-h-screen w-full bg-background-light-secondary">
    {@render children()}
  </main>
</QueryClientProvider>