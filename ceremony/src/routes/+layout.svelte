<script lang="ts">
import Navbar from "$lib/layout/Navbar/index.svelte"
import { beforeNavigate, goto } from "$app/navigation"
import { checkAuth, type SessionError } from "$lib/utils/auth.ts"
import { supabase } from "$lib/supabase/client.ts"
import { user } from "$lib/stores/user.svelte.ts"
import { Toaster } from "svelte-sonner"
import { QueryClient, QueryClientProvider } from "@tanstack/svelte-query"
import {onMount} from "svelte";
import {Application} from "@splinetool/runtime";

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

let canvas
let app
let model
let loading = true

onMount(() => {
  const canvas = document.getElementById("canvas3d");
  if (!canvas) return;
  app = new Application(canvas);
  if (!app) return;
  app.load("https://draft.spline.design/r6WgY2-52aHVU2TZ/scene.splinecode").then((splineScene) => {
    console.log("hello");
    model = splineScene;
    loading = false;
    console.log(loading);
  });
})

const queryClient = new QueryClient()
</script>

<Toaster position="bottom-right" toastOptions={{ class: 'rounded-none border border-black',}}/>

<QueryClientProvider client={queryClient}>
  <Navbar/>
  <canvas id="canvas3d" class="pointer-events-none w-full h-auto inset-0 absolute -z-10"></canvas>
  <main class="flex flex-col items-center justify-center min-h-screen w-full bg-background-light-secondary">
    {@render children()}
  </main>
</QueryClientProvider>



