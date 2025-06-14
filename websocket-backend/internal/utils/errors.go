package utils

import (
	"fmt"
	"runtime"
	"strings"
	"time"
)

// ErrorType represents different categories of errors
type ErrorType string

const (
	ErrorTypeValidation ErrorType = "VALIDATION"
	ErrorTypeNetwork    ErrorType = "NETWORK"
	ErrorTypeGraphQL    ErrorType = "GRAPHQL"
	ErrorTypeWebSocket  ErrorType = "WEBSOCKET"
	ErrorTypeInternal   ErrorType = "INTERNAL"
	ErrorTypeConfig     ErrorType = "CONFIG"
	ErrorTypeTimeout    ErrorType = "TIMEOUT"
)

// AppError represents a structured application error
type AppError struct {
	Type        ErrorType              `json:"type"`
	Code        string                 `json:"code"`
	Message     string                 `json:"message"`
	Details     string                 `json:"details,omitempty"`
	Cause       error                  `json:"-"` // Don't serialize the underlying error
	Context     map[string]interface{} `json:"context,omitempty"`
	Timestamp   time.Time              `json:"timestamp"`
	StackTrace  string                 `json:"stackTrace,omitempty"`
	Retryable   bool                   `json:"retryable"`
	Component   string                 `json:"component"`
}

// Error implements the error interface
func (e *AppError) Error() string {
	if e.Details != "" {
		return fmt.Sprintf("[%s:%s] %s: %s", e.Type, e.Code, e.Message, e.Details)
	}
	return fmt.Sprintf("[%s:%s] %s", e.Type, e.Code, e.Message)
}

// Unwrap returns the underlying error for error unwrapping
func (e *AppError) Unwrap() error {
	return e.Cause
}

// IsRetryable returns whether the error is retryable
func (e *AppError) IsRetryable() bool {
	return e.Retryable
}

// WithContext adds context to the error
func (e *AppError) WithContext(key string, value interface{}) *AppError {
	if e.Context == nil {
		e.Context = make(map[string]interface{})
	}
	e.Context[key] = value
	return e
}

// WithDetails adds additional details to the error
func (e *AppError) WithDetails(details string) *AppError {
	e.Details = details
	return e
}

// NewAppError creates a new application error
func NewAppError(errorType ErrorType, code, message, component string) *AppError {
	return &AppError{
		Type:      errorType,
		Code:      code,
		Message:   message,
		Component: component,
		Timestamp: time.Now().UTC(),
		Context:   make(map[string]interface{}),
	}
}

// WrapError wraps an existing error with application error context
func WrapError(err error, errorType ErrorType, code, message, component string) *AppError {
	appErr := NewAppError(errorType, code, message, component)
	appErr.Cause = err
	
	// Capture stack trace
	if includeStackTrace {
		appErr.StackTrace = getStackTrace()
	}
	
	return appErr
}

// Configuration for error handling
var (
	includeStackTrace = false // Set to true in development
)

// SetIncludeStackTrace configures whether to include stack traces in errors
func SetIncludeStackTrace(include bool) {
	includeStackTrace = include
}

// getStackTrace captures the current stack trace
func getStackTrace() string {
	const depth = 32
	var pcs [depth]uintptr
	n := runtime.Callers(3, pcs[:]) // Skip getStackTrace, WrapError, and the calling function
	
	var trace strings.Builder
	frames := runtime.CallersFrames(pcs[:n])
	
	for {
		frame, more := frames.Next()
		trace.WriteString(fmt.Sprintf("%s:%d %s\n", frame.File, frame.Line, frame.Function))
		if !more {
			break
		}
	}
	
	return trace.String()
}

// Predefined error constructors for common scenarios

// ValidationError creates a validation error
func ValidationError(code, message, component string) *AppError {
	return NewAppError(ErrorTypeValidation, code, message, component)
}

// NetworkError creates a network error (usually retryable)
func NetworkError(code, message, component string) *AppError {
	err := NewAppError(ErrorTypeNetwork, code, message, component)
	err.Retryable = true
	return err
}

// GraphQLError creates a GraphQL error
func GraphQLError(code, message, component string) *AppError {
	return NewAppError(ErrorTypeGraphQL, code, message, component)
}

// WebSocketError creates a WebSocket error
func WebSocketError(code, message, component string) *AppError {
	return NewAppError(ErrorTypeWebSocket, code, message, component)
}

// InternalError creates an internal error
func InternalError(code, message, component string) *AppError {
	return NewAppError(ErrorTypeInternal, code, message, component)
}

// ConfigError creates a configuration error
func ConfigError(code, message, component string) *AppError {
	return NewAppError(ErrorTypeConfig, code, message, component)
}

// TimeoutError creates a timeout error (usually retryable)
func TimeoutError(code, message, component string) *AppError {
	err := NewAppError(ErrorTypeTimeout, code, message, component)
	err.Retryable = true
	return err
}

// Error handling utilities

// IsRetryableError checks if an error is retryable
func IsRetryableError(err error) bool {
	if appErr, ok := err.(*AppError); ok {
		return appErr.IsRetryable()
	}
	
	// Check for common retryable error patterns
	errStr := strings.ToLower(err.Error())
	retryablePatterns := []string{
		"timeout",
		"connection refused",
		"connection reset",
		"temporary failure",
		"service unavailable",
		"too many requests",
	}
	
	for _, pattern := range retryablePatterns {
		if strings.Contains(errStr, pattern) {
			return true
		}
	}
	
	return false
}

// GetErrorType extracts the error type from an error
func GetErrorType(err error) ErrorType {
	if appErr, ok := err.(*AppError); ok {
		return appErr.Type
	}
	return ErrorTypeInternal
}

// GetErrorCode extracts the error code from an error
func GetErrorCode(err error) string {
	if appErr, ok := err.(*AppError); ok {
		return appErr.Code
	}
	return "UNKNOWN"
}

// LogError logs an error with appropriate context
func LogError(err error, logger *Logger, additionalContext ...map[string]interface{}) {
	if appErr, ok := err.(*AppError); ok {
		fields := make(map[string]interface{})
		
		// Add error fields
		fields["errorType"] = appErr.Type
		fields["errorCode"] = appErr.Code
		fields["retryable"] = appErr.Retryable
		fields["component"] = appErr.Component
		
		// Add error context
		for k, v := range appErr.Context {
			fields[k] = v
		}
		
		// Add additional context
		if len(additionalContext) > 0 {
			for k, v := range additionalContext[0] {
				fields[k] = v
			}
		}
		
		// Add stack trace if available
		if appErr.StackTrace != "" {
			fields["stackTrace"] = appErr.StackTrace
		}
		
		logger.Error(appErr.Message, fields)
	} else {
		// Handle regular errors
		fields := make(map[string]interface{})
		fields["errorType"] = "UNKNOWN"
		
		if len(additionalContext) > 0 {
			for k, v := range additionalContext[0] {
				fields[k] = v
			}
		}
		
		logger.Error(err.Error(), fields)
	}
}

// RecoverPanic recovers from panics and converts them to errors
func RecoverPanic(component string) error {
	if r := recover(); r != nil {
		err := InternalError("PANIC", fmt.Sprintf("Panic recovered: %v", r), component)
		if includeStackTrace {
			err.StackTrace = getStackTrace()
		}
		return err
	}
	return nil
}

// SafeExecute executes a function and recovers from panics
func SafeExecute(fn func() error, component string) error {
	defer func() {
		if err := RecoverPanic(component); err != nil {
			LogError(err, ServerLogger)
		}
	}()
	
	return fn()
}

// RetryConfig represents retry configuration
type RetryConfig struct {
	MaxAttempts int
	BaseDelay   time.Duration
	MaxDelay    time.Duration
	Multiplier  float64
}

// DefaultRetryConfig returns a default retry configuration
func DefaultRetryConfig() RetryConfig {
	return RetryConfig{
		MaxAttempts: 3,
		BaseDelay:   100 * time.Millisecond,
		MaxDelay:    5 * time.Second,
		Multiplier:  2.0,
	}
}

// RetryWithBackoff executes a function with exponential backoff retry
func RetryWithBackoff(fn func() error, config RetryConfig, component string) error {
	var lastErr error
	
	for attempt := 1; attempt <= config.MaxAttempts; attempt++ {
		err := fn()
		if err == nil {
			return nil
		}
		
		lastErr = err
		
		// Don't retry if error is not retryable
		if !IsRetryableError(err) {
			break
		}
		
		// Don't sleep after the last attempt
		if attempt == config.MaxAttempts {
			break
		}
		
		// Calculate delay with exponential backoff
		delay := time.Duration(float64(config.BaseDelay) * 
			(config.Multiplier * float64(attempt-1)))
		if delay > config.MaxDelay {
			delay = config.MaxDelay
		}
		
		ServerLogger.Warn("Retrying operation", map[string]interface{}{
			"attempt":   attempt,
			"maxAttempts": config.MaxAttempts,
			"delay":     delay.String(),
			"error":     err.Error(),
			"component": component,
		})
		
		time.Sleep(delay)
	}
	
	return WrapError(lastErr, ErrorTypeInternal, "RETRY_EXHAUSTED", 
		fmt.Sprintf("Operation failed after %d attempts", config.MaxAttempts), component)
} 