<script lang="ts">
  import { type Snippet } from "svelte";
  import Navbar from "$lib/components/Navbar/index.svelte";
  import { Toaster } from 'svelte-french-toast';
  import { supabase } from "$lib/supabase.ts";
  import { userSession } from "$lib/stores/session.ts";
  import { goto } from "$app/navigation";

  import "../styles/tailwind.css";
  import "../styles/fonts.css";
  import {page} from "$app/stores";

  let {children}: { children: Snippet } = $props();

  let loading = $state(true);

  $effect(() => {
    supabase.auth.getSession().then(({data: {session}}) => {
      if (!session) {
        if($page.url.pathname !== '/auth/login' && $page.url.pathname !== '/auth/register') {
          goto('/auth/login');
        }
      }
    });

    const {data: {subscription}} = supabase.auth.onAuthStateChange((event, session) => {
      userSession.set(session);
      if (!session) {
        if($page.url.pathname !== '/auth/login' && $page.url.pathname !== '/auth/register') {
          goto('/auth/login');
        }
      }
    });

    return () => {
      subscription.unsubscribe();
    };
  });

</script>

<Toaster />
<Navbar/>
{@render children()}

