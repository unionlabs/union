<script lang="ts">
  import {type Snippet} from "svelte";
  import Navbar from "$lib/layout/Navbar/index.svelte";
  import {beforeNavigate, goto} from "$app/navigation";
  import {checkAuth, type SessionError} from "$lib/utils/auth.ts";
  import {supabase} from "$lib/supabase/client.ts";
  import {user} from "$lib/stores/user.svelte.ts";

  import "../styles/tailwind.css";
  import {Toaster} from "svelte-sonner";

  let {children} = $props();

  beforeNavigate(async ({from, to, cancel}) => {
    const pathname = to?.route?.id;
    if (pathname) {
      const segments = pathname.split('/').filter(Boolean);
      if (segments[0] === 'app') {
        const authCheck = await checkAuth();

        authCheck.match(
          () => {
          },
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
      user.session = session
    });
    return () => {
      subscription.unsubscribe();
    };
  })

</script>

<Toaster
        position="bottom-right"
        toastOptions={{
          class: 'rounded-none border border-black',
	}}
/>

<Navbar/>
<main class="flex flex-col items-center justify-center min-h-screen w-full bg-background-light-secondary">
  {@render children()}
</main>

