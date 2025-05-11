import type { PostgrestError } from "@supabase/supabase-js";
import { Data } from "effect";

export class AuthenticationError extends Data.TaggedError("AuthenticationError")<{
  cause: unknown;
}> {}

export class SupabaseClientError extends Data.TaggedError("SupabaseClientError")<{
  cause: unknown;
}> {}

export class SupabaseError extends Data.TaggedError("SupabaseError")<{
  error?: PostgrestError;
  cause?: unknown;
}> {}

export class UnknownException extends Data.TaggedError("UnknownException")<{
  message: string;
  cause?: unknown;
}> {}

export class AchievementError extends Data.TaggedError("AchievementError")<{
  cause: unknown;
  operation: "load" | "loadAvailable";
}> {}

export class LeaderboardError extends Data.TaggedError("LeaderboardError")<{
  cause: unknown;
  operation: "load";
}> {}

export class MissionError extends Data.TaggedError("MissionError")<{
  cause: unknown;
  operation: "load" | "loadAvailable";
}> {}

export class RewardError extends Data.TaggedError("RewardError")<{
  cause: unknown;
  operation: "load" | "loadAvailable";
}> {}

export class WalletError extends Data.TaggedError("WalletError")<{
  cause: unknown;
  operation: "add" | "remove" | "load";
}> {}
