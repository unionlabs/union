<script lang="ts">
import { getContributions } from "$lib/supabase"
import Spinner from "$lib/components/Spinner.svelte"
import Text from "$lib/components/typography/Text.svelte"
import Blink from "$lib/components/Blink.svelte"
import H4 from "$lib/components/typography/H4.svelte"

let intervalId: NodeJS.Timeout | number
let contributions = $state([
  {
    public_key_hash: "174874cebccadc3e169c8826b00171268200050e428427d0757937d2cf407d36",
    user_name: "Lukas",
    avatar_url: "https://avatars.githubusercontent.com/u/36674091?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "174874cebccadc3e169c8826b00171268200050e428427d0757937d2cf407d36",
    user_name: "Lukas",
    avatar_url: "https://avatars.githubusercontent.com/u/36674091?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "174874cebccadc3e169c8826b00171268200050e428427d0757937d2cf407d36",
    user_name: "Lukas",
    avatar_url: "https://avatars.githubusercontent.com/u/36674091?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "bf75d0b4d76e7ffa96a532109dd6fd6a57a0f76b0cbd5492b6db2caccbcc1ae5",
    user_name: "ben",
    avatar_url: "https://avatars.githubusercontent.com/u/57334811?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "174874cebccadc3e169c8826b00171268200050e428427d0757937d2cf407d36",
    user_name: "Lukas",
    avatar_url: "https://avatars.githubusercontent.com/u/36674091?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "174874cebccadc3e169c8826b00171268200050e428427d0757937d2cf407d36",
    user_name: "Lukas",
    avatar_url: "https://avatars.githubusercontent.com/u/36674091?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "174874cebccadc3e169c8826b00171268200050e428427d0757937d2cf407d36",
    user_name: "Lukas",
    avatar_url: "https://avatars.githubusercontent.com/u/36674091?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "bf75d0b4d76e7ffa96a532109dd6fd6a57a0f76b0cbd5492b6db2caccbcc1ae5",
    user_name: "ben",
    avatar_url: "https://avatars.githubusercontent.com/u/57334811?v=4"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  },
  {
    public_key_hash: "39945feba9bcc2ee19861935144a5a71df572d73702eb84b24427810c9707200",
    user_name: "Hussein AIT LAHCEN",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocKJJ_6Rtsc9qEp8-tMsR3cnxXm2YZbo2TCQslR9THnHbaL6=s96-c"
  },
  {
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  },
  {
    public_key_hash: "98442419f2e7cf932fe001081a293ed19966aa29ef50353586f100cf1d524d8f",
    user_name: "Kaylei Fleming",
    avatar_url:
      "https://lh3.googleusercontent.com/a/ACg8ocK2e14tsvQf67Pkds0U6kXNdpzfzOYv1BsZgHCoiWeUBvcEeg=s96-c"
  }
])

function loadContributions() {
  contributions.push({
    public_key_hash: "e8feb4e87a8dba95f1b6b07c9d856a0f3e079769cafc497ea1e9cc9c239f6f0c",
    user_name: "cor",
    avatar_url: "https://avatars.githubusercontent.com/u/4728062?v=4"
  })
  // contributions = await getContributions();
}

$effect(() => {
  intervalId = setInterval(loadContributions, 1000 * 5)

  return () => {
    if (intervalId) clearInterval(intervalId)
  }
})
</script>

{#if contributions}
  <div class="flex flex-col-reverse items-center h-svh overflow-y-auto pb-24 pt-36 w-full">
    <div class="w-full h-48 bg-gradient-to-b via-black from-black to-transparent absolute top-0"></div>
    <div class="flex flex-col items-center max-w-sm">
      {#each contributions as contribution, index }
        {#if index !== 0}
          <div class="h-4 w-[2px] bg-white"></div>
        {/if}
        <a href="/contributions/{contribution.public_key_hash}"
           class="text-white flex gap-1 items-center border-white border px-3 py-2 w-full">
          <img class="size-7" src={contribution.avatar_url} alt="">
          <Text class="uppercase max-w-48 truncate">{contribution.user_name}</Text>
        </a>
      {/each}
      <div class="h-4 w-[2px] bg-white"></div>
      <div class="text-white flex gap-2 items-center border-white border px-3 py-2 mb-16">
        <Spinner class="size-7 text-union-accent-500"/>
        <span>Next contribution...</span>
      </div>
      <H4>
        <Blink/>
      </H4>
    </div>
  </div>
{:else}
  <Spinner class="size-5 text-union-accent-500"/>
{/if}