<script lang="ts">
  import {type Snippet} from "svelte";
  import Navbar from "$lib/components/Navbar/index.svelte";
  import {Toaster} from 'svelte-french-toast';
  import {supabase} from "$lib/supabase.ts";
  import {userSession} from "$lib/stores/session.ts";

  import "../styles/tailwind.css";
  import "../styles/fonts.css";

  let {children}: { children: Snippet } = $props();

  $effect(() => {
    supabase.auth.getSession().then(({data: {session}}) => {
      userSession.set(session);
    });

    const {data: {subscription}} = supabase.auth.onAuthStateChange((event, session) => {
      userSession.set(session);
    });

    return () => {
      subscription.unsubscribe();
    };
  });

</script>

<Toaster/>
<Navbar/>
<main class="flex flex-col items-center justify-center min-h-screen w-full bg-background-light-secondary">
  {@render children()}
</main>

