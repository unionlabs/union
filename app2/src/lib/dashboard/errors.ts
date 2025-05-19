import type { PostgrestError } from "@supabase/supabase-js"
import { Data } from "effect"

export class AuthenticationError extends Data.TaggedError("AuthenticationError")<{
  cause: unknown
  code?: string
  message?: string
}> {}

export class ProviderLinkError extends Data.TaggedError("ProviderLinkError")<{
  provider: string
  message: string
}> {}

export class EmailLinkError extends Data.TaggedError("EmailLinkError")<{
  email: string
  message: string
}> {}

export class SupabaseClientError extends Data.TaggedError("SupabaseClientError")<{
  cause: unknown
}> {}

export class SupabaseError extends Data.TaggedError("SupabaseError")<{
  error?: PostgrestError
  cause?: unknown
}> {}

export class DashboardUnknownException extends Data.TaggedError("DashboardUnknownException")<{
  message: string
  cause?: unknown
}> {}

export class AchievementError extends Data.TaggedError("AchievementError")<{
  cause: unknown
  operation: "load" | "loadAvailable"
}> {}

export class LeaderboardError extends Data.TaggedError("LeaderboardError")<{
  cause: unknown
  operation: "load" | "loadLevels"
}> {}

export class MissionError extends Data.TaggedError("MissionError")<{
  cause: unknown
  operation: "load" | "loadAvailable"
}> {}

export class RewardError extends Data.TaggedError("RewardError")<{
  cause: unknown
  operation: "load" | "loadAvailable"
}> {}

export class WalletError extends Data.TaggedError("WalletError")<{
  cause: unknown
  operation: "add" | "remove" | "load" | "loadChains"
}> {}

export class ChainError extends Data.TaggedError("ChainError")<{
  cause: unknown
  operation: "load"
}> {}

export class CategoryError extends Data.TaggedError("CategoryError")<{
  cause: unknown
  operation: "load"
}> {}

// Helper to map Supabase error codes to our error types
export const createAuthError = (error: string, errorDescription?: string) => {
  switch (error) {
    case "provider-link-error":
      return new ProviderLinkError({
        provider: errorDescription?.split(" ")[0] || "unknown",
        message: errorDescription || "Provider is already linked to another account",
      })
    case "email-link-error":
      return new EmailLinkError({
        email: errorDescription?.split(" ")[0] || "unknown",
        message: errorDescription || "Email is already linked to another account",
      })
    case "unauthorized":
      return new AuthenticationError({
        cause: error,
        code: error,
        message: errorDescription || "Authentication failed",
      })
    default:
      return new AuthenticationError({
        cause: error,
        code: error,
        message: errorDescription || "An unknown authentication error occurred",
      })
  }
}
