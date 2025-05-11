import { browser } from "$app/environment";
import { dashboard } from "$lib/dashboard/stores/user.svelte";
import { Effect, pipe, Option } from "effect";
import type { Handle } from "@sveltejs/kit";

const PROTECTED_PATHS = ["/dashboard"];

export const handle: Handle = async ({ event, resolve }) => {
  if (browser && PROTECTED_PATHS.some(path => event.url.pathname.startsWith(path))) {
    return Effect.runPromise(
      pipe(
        Effect.succeed(dashboard.session),
        Effect.flatMap(session => 
          Option.isNone(session)
            ? Effect.succeed(new Response("Redirect", {
                status: 302,
                headers: { Location: "/" }
              }))
            : Effect.succeed(resolve(event))
        )
      )
    );
  }

  return resolve(event);
};
