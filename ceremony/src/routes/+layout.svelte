<script lang="ts">
  import {type Snippet} from "svelte";
  import Navbar from "$lib/components/Navbar/index.svelte";
  import {Toaster} from 'svelte-french-toast';
  import {beforeNavigate, goto} from "$app/navigation";
  import {checkAuth, type SessionError} from "$lib/utils/auth.ts";
  import {supabase} from "$lib/supabase.ts";
  import {userSession} from "$lib/stores/session.ts";

  import "../styles/tailwind.css";
  import "../styles/fonts.css";


  let { children }: { children: Snippet } = $props();

  beforeNavigate(async ({ from, to, cancel }) => {
    const pathname = to?.route?.id;
    if (pathname) {
      const segments = pathname.split('/').filter(Boolean);
      if (segments[0] === 'app') {
        const authCheck = await checkAuth();

        authCheck.match(
          () => {},
          (error: SessionError) => {
            console.error(error.message);
            cancel();
            goto('/auth/register');
          }
        );
      }
    }
  });

  $effect(() => {
    const {data: {subscription}} = supabase.auth.onAuthStateChange((event, session) => {
      userSession.set(session);
    });
    return () => {
      subscription.unsubscribe();
    };
  })

</script>

<Toaster/>
<Navbar/>
<main class="flex flex-col items-center justify-center min-h-screen w-full bg-background-light-secondary">
  {@render children()}
</main>

