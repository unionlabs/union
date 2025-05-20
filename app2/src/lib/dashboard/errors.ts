import type { PostgrestError } from "@supabase/supabase-js"
import { Data } from "effect"

export type BaseErrorArgs = {
  cause: unknown
  operation: string
  message?: string
}

export class SupabaseClientError extends Data.TaggedError("SupabaseClientError")<BaseErrorArgs> {}

export class ChainError extends Data.TaggedError("ChainError")<BaseErrorArgs> {}

export class CategoryError extends Data.TaggedError("CategoryError")<BaseErrorArgs> {}

export class SupabaseError extends Data.TaggedError("SupabaseError")<
  BaseErrorArgs & {
    error?: PostgrestError
  }
> {}

export class DashboardUnknownException
  extends Data.TaggedError("DashboardUnknownException")<BaseErrorArgs>
{}

export class AchievementError extends Data.TaggedError("AchievementError")<BaseErrorArgs> {}

export class LeaderboardError extends Data.TaggedError("LeaderboardError")<BaseErrorArgs> {}

export class MissionError extends Data.TaggedError("MissionError")<BaseErrorArgs> {}

export class RewardError extends Data.TaggedError("RewardError")<BaseErrorArgs> {}

export class WalletError extends Data.TaggedError("WalletError")<BaseErrorArgs> {}

export class ProviderLinkError extends Data.TaggedError("ProviderLinkError")<
  BaseErrorArgs & {
    provider?: string
  }
> {}

export class EmailLinkError extends Data.TaggedError("EmailLinkError")<
  BaseErrorArgs & {
    email?: string
  }
> {}

export class AccountError extends Data.TaggedError("AccountError")<BaseErrorArgs> {}

export class AuthenticationError extends Data.TaggedError("AuthenticationError")<BaseErrorArgs> {}

export class ExperienceError extends Data.TaggedError("ExperienceError")<BaseErrorArgs> {}

// Helper to map Supabase error codes to our error types
export function mapSupabaseErrorToCustomError(
  errorCode: string,
  errorDescription?: string,
): Error {
  switch (errorCode) {
    case "provider-link-error":
      return new ProviderLinkError({
        operation: "link",
        cause: errorDescription,
        provider: errorDescription?.split(" ")[0] || "unknown",
        message: errorDescription || "Provider is already linked to another account",
      })
    case "email-link-error":
      return new EmailLinkError({
        operation: "link",
        cause: errorDescription,
        email: errorDescription?.split(" ")[0] || "unknown",
        message: errorDescription || "Email is already linked to another account",
      })
    default:
      return new Error(errorDescription || "Unknown error occurred")
  }
}
