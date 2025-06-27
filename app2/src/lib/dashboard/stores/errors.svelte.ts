import type {
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
  SnagAPIError,
  SnagAuthenticationError,
  SnagBadRequestError,
  SnagClientError,
  SnagConnectionError,
  SnagError,
  SnagInternalServerError,
  SnagNotFoundError,
  SnagPermissionDeniedError,
  SnagRateLimitError,
  SnagUnprocessableEntityError,
  SnagUserError,
  SupabaseClientError,
  SupabaseError,
  WalletError,
} from "$lib/dashboard/errors"

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
  | SnagError
  | SnagUserError
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
