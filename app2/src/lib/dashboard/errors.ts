import type { PostgrestError } from "@supabase/supabase-js"
import { Data } from "effect"

interface BaseErrorArgs {
  operation: string
  message?: string
  cause?: unknown
}

export class AuthenticationError extends Data.TaggedError("AuthenticationError")<BaseErrorArgs> {}

export class ProviderLinkError extends Data.TaggedError("ProviderLinkError")<BaseErrorArgs & {
  provider: string
}> {}

export class EmailLinkError extends Data.TaggedError("EmailLinkError")<BaseErrorArgs & {
  email: string
}> {}

export class SupabaseClientError extends Data.TaggedError("SupabaseClientError")<BaseErrorArgs> {}

export class SupabaseError extends Data.TaggedError("SupabaseError")<BaseErrorArgs & {
  error?: PostgrestError
}> {}

export class DashboardUnknownException extends Data.TaggedError("DashboardUnknownException")<BaseErrorArgs> {}

export class AchievementError extends Data.TaggedError("AchievementError")<BaseErrorArgs> {}

export class LeaderboardError extends Data.TaggedError("LeaderboardError")<BaseErrorArgs> {}

export class MissionError extends Data.TaggedError("MissionError")<BaseErrorArgs> {}

export class RewardError extends Data.TaggedError("RewardError")<BaseErrorArgs> {}

export class WalletError extends Data.TaggedError("WalletError")<BaseErrorArgs> {}

export class ChainError extends Data.TaggedError("ChainError")<BaseErrorArgs> {}

export class CategoryError extends Data.TaggedError("CategoryError")<BaseErrorArgs> {}

export class AccountError extends Data.TaggedError("AccountError")<BaseErrorArgs> {}

// Helper to map Supabase error codes to our error types
export const createAuthError = (error: string, errorDescription?: string) => {
  switch (error) {
    case "provider-link-error":
      return new ProviderLinkError({
        operation: "link",
        provider: errorDescription?.split(" ")[0] || "unknown",
        message: errorDescription || "Provider is already linked to another account",
      })
    case "email-link-error":
      return new EmailLinkError({
        operation: "link",
        email: errorDescription?.split(" ")[0] || "unknown",
        message: errorDescription || "Email is already linked to another account",
      })
    case "unauthorized":
      return new AuthenticationError({
        operation: "authenticate",
        message: errorDescription || "Authentication failed",
        cause: error,
      })
    default:
      return new AuthenticationError({
        operation: "authenticate",
        message: errorDescription || "An unknown authentication error occurred",
        cause: error,
      })
  }
}
