import type { User } from "@supabase/supabase-js";
import type { AuthProvider } from "./authentication";
import { Option, pipe, Effect} from "effect";
import type { PostgrestSingleResponse, SupabaseClient } from '@supabase/supabase-js';
import { SupabaseError, AuthenticationError } from './errors';
import { getSupabaseClient } from './client';
  

export const hasProviderLinked = (user: User, provider: AuthProvider) =>
  user.identities?.some(identity => identity.provider === provider) ?? false;

export const getProviderId = (user: User, provider: AuthProvider) =>
  pipe(
    user.identities,
    Option.fromNullable,
    Option.flatMap(identities =>
      Option.fromNullable(identities.find(id => id.provider.toLowerCase() === provider.toLowerCase()))
    ),
    Option.map(identity => identity.id)
  );

export const isProviderConnected = (user: User, provider: AuthProvider) =>
  pipe(
    user.identities,
    Option.fromNullable,
    Option.map(identities =>
      identities.some(id => id.provider.toLowerCase() === provider.toLowerCase())
    ),
    Option.getOrElse(() => false)
  );

export function queryEffect<A>(
    supabaseCall: () => PromiseLike<PostgrestSingleResponse<A>>
  ): Effect.Effect<unknown, SupabaseError, A> {
    return Effect.tryPromise({
      try: async () => Promise.resolve(await supabaseCall()),
      catch: (error) => new SupabaseError({ cause: error }),
    }).pipe(
      Effect.flatMap(({ data, error }) =>
        error || !data
          ? Effect.fail(new SupabaseError({ cause: error ?? "No data returned" }))
          : Effect.succeed(data)
      )
    );
  }

  export const querySupabase = Effect.gen(function* () {
    const client = yield* getSupabaseClient();
  
    return <A>(
      cb: (client: SupabaseClient) => PromiseLike<PostgrestSingleResponse<A>>
    ): Effect.Effect<unknown, SupabaseError, A> => {
      return queryEffect(() => cb(client));
    };
  });

export const requireAuthenticatedUserId = (
  user: unknown
): Effect.Effect<unknown, AuthenticationError, string> => {
  return pipe(
    Option.fromNullable((user as any)?.session?.user?.id),
    Option.match({
      onNone: () =>
        Effect.fail(
          new AuthenticationError({ cause: "User is not authenticated" })
        ),
      onSome: (userId) => Effect.succeed(userId),
    })
  );
};

export const getChains = (user: unknown) =>
    Effect.gen(function* () {
      const userId = yield* requireAuthenticatedUserId(user);
  
      const runQuery = yield* querySupabase;
      const chains = yield* runQuery((client) =>
        client.from("chains").select("*").eq("user_id", userId)
      );
      
      return chains;
});
  
