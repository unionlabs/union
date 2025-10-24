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

export class BTCFIError extends Data.TaggedError("BTCFIError")<BaseErrorArgs> {}

// Helper to map Supabase error codes to our error types
export function mapSupabaseErrorToCustomError(
  errorCode: string,
  errorDescription?: string,
): DashboardUnknownException | ProviderLinkError | EmailLinkError {
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
    // Twitter OAuth specific errors that come back in URL
    case "access_denied":
      return new ProviderLinkError({
        operation: "link",
        cause: errorDescription,
        provider: "twitter",
        message:
          "You cancelled the Twitter connection. Please try again if you'd like to connect your account.",
      })
    case "user_cancelled":
      return new ProviderLinkError({
        operation: "link",
        cause: errorDescription,
        provider: "twitter",
        message:
          "You cancelled the Twitter connection. Please try again if you'd like to connect your account.",
      })
    case "oauth_problem":
      return new ProviderLinkError({
        operation: "link",
        cause: errorDescription,
        provider: "twitter",
        message: "There was a problem connecting to Twitter. Please try again.",
      })
    case "invalid_request":
      return new ProviderLinkError({
        operation: "link",
        cause: errorDescription,
        provider: "twitter",
        message: "Invalid authentication request. Please try connecting again.",
      })
    case "temporarily_unavailable":
      return new DashboardUnknownException({
        operation: "auth",
        cause: errorDescription,
        message:
          "Twitter authentication is temporarily unavailable. Please try again in a few minutes.",
      })
    case "unsupported_response_type":
      return new DashboardUnknownException({
        operation: "auth",
        cause: errorDescription,
        message: "Authentication configuration error. Please contact support.",
      })
    case "server_error":
      return new DashboardUnknownException({
        operation: "auth",
        cause: errorDescription,
        message: "Server error occurred during authentication. Please try again later.",
      })
    // Supabase specific errors for identity linking
    case "identity_already_exists":
      return new ProviderLinkError({
        operation: "link",
        cause: errorDescription,
        provider: "twitter",
        message:
          "This Twitter account is already linked to another user. Please try a different account.",
      })
    case "signup_disabled":
      return new DashboardUnknownException({
        operation: "auth",
        cause: errorDescription,
        message: "Account registration is currently disabled. Please try again later.",
      })
    default:
      // Check if error description contains common patterns
      if (
        errorDescription?.toLowerCase().includes("already linked")
        || errorDescription?.toLowerCase().includes("identity is already linked")
      ) {
        return new ProviderLinkError({
          operation: "link",
          cause: errorDescription,
          provider: "twitter",
          message:
            "This Twitter account is already linked to another user. Please try a different account.",
        })
      }

      if (
        errorDescription?.toLowerCase().includes("error getting user email")
        || errorDescription?.toLowerCase().includes("email not verified")
        || errorDescription?.toLowerCase().includes("no email address")
      ) {
        return new ProviderLinkError({
          operation: "link",
          cause: errorDescription,
          provider: "twitter",
          message:
            "Your Twitter account doesn't have a verified email address. Please add and verify an email in your Twitter settings, then try again.",
        })
      }

      if (
        errorDescription?.toLowerCase().includes("callback url not approved")
        || errorDescription?.toLowerCase().includes("callback url")
      ) {
        return new DashboardUnknownException({
          operation: "auth",
          cause: errorDescription,
          message: "Authentication configuration error. Please contact support.",
        })
      }

      if (
        errorDescription?.toLowerCase().includes("rate limit")
        || errorDescription?.toLowerCase().includes("too many requests")
      ) {
        return new DashboardUnknownException({
          operation: "auth",
          cause: errorDescription,
          message:
            "Too many authentication attempts. Please wait a few minutes before trying again.",
        })
      }

      if (
        errorDescription?.toLowerCase().includes("user suspended")
        || errorDescription?.toLowerCase().includes("account suspended")
      ) {
        return new ProviderLinkError({
          operation: "link",
          cause: errorDescription,
          provider: "twitter",
          message: "This Twitter account has been suspended. Please use a different account.",
        })
      }

      return new DashboardUnknownException({
        operation: "auth",
        cause: errorDescription,
        message: errorDescription || "Unknown error occurred",
      })
  }
}
