import {
  AccountError,
  AchievementError,
  AuthenticationError,
  CategoryError,
  ChainError,
  DashboardUnknownException,
  EmailLinkError,
  ExperienceError,
  LeaderboardError,
  MissionError,
  ProviderLinkError,
  RewardError,
  SupabaseClientError,
  SupabaseError,
  WalletError,
} from "$lib/dashboard/errors"
import {
  SnagAPIError,
  SnagAuthenticationError,
  SnagBadRequestError,
  SnagClientError,
  SnagConnectionError,
  SnagInternalServerError,
  SnagNotFoundError,
  SnagPermissionDeniedError,
  SnagRateLimitError,
  SnagUnprocessableEntityError,
} from "../snag/errors"

type DashboardError =
  | AuthenticationError
  | SupabaseClientError
  | ChainError
  | CategoryError
  | SupabaseError
  | DashboardUnknownException
  | AchievementError
  | LeaderboardError
  | MissionError
  | RewardError
  | WalletError
  | ProviderLinkError
  | EmailLinkError
  | AccountError
  | ExperienceError
  | SnagClientError
  | SnagAPIError
  | SnagBadRequestError
  | SnagAuthenticationError
  | SnagPermissionDeniedError
  | SnagNotFoundError
  | SnagUnprocessableEntityError
  | SnagRateLimitError
  | SnagInternalServerError
  | SnagConnectionError

class ErrorStore {
  current: DashboardError | null = $state(null)

  showError(error: DashboardError) {
    this.current = error
  }

  clearError() {
    this.current = null
  }
}

export const errorStore = new ErrorStore()
export type { DashboardError }
