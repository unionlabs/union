package config

import (
	"fmt"
	"net/url"
	"strconv"
)

// This file contains ONLY validation logic for configurations created by config.New()
// All configuration values are hardcoded in config.go - no external files or env vars

// ValidationError represents a configuration validation error
type ValidationError struct {
	Field   string
	Value   interface{}
	Message string
}

func (e ValidationError) Error() string {
	return fmt.Sprintf("config validation error for field '%s' (value: %v): %s", e.Field, e.Value, e.Message)
}

// ValidationErrors represents multiple validation errors
type ValidationErrors []ValidationError

func (e ValidationErrors) Error() string {
	if len(e) == 0 {
		return "no validation errors"
	}
	
	msg := fmt.Sprintf("found %d configuration validation errors:\n", len(e))
	for i, err := range e {
		msg += fmt.Sprintf("  %d. %s\n", i+1, err.Error())
	}
	return msg
}

// Validate validates the configuration and returns any validation errors
func (c *Config) Validate() error {
	var errors ValidationErrors
	
	// Validate Port
	if c.Port == "" {
		errors = append(errors, ValidationError{
			Field:   "PORT",
			Value:   c.Port,
			Message: "port cannot be empty",
		})
	} else {
		if port, err := strconv.Atoi(c.Port); err != nil {
			errors = append(errors, ValidationError{
				Field:   "PORT",
				Value:   c.Port,
				Message: "port must be a valid integer",
			})
		} else if port < 1 || port > 65535 {
			errors = append(errors, ValidationError{
				Field:   "PORT",
				Value:   port,
				Message: "port must be between 1 and 65535",
			})
		}
	}
	
	// Validate GraphQL Endpoint
	if c.GraphQLEndpoint == "" {
		errors = append(errors, ValidationError{
			Field:   "GRAPHQL_ENDPOINT",
			Value:   c.GraphQLEndpoint,
			Message: "GraphQL endpoint cannot be empty",
		})
	} else {
		if _, err := url.Parse(c.GraphQLEndpoint); err != nil {
			errors = append(errors, ValidationError{
				Field:   "GRAPHQL_ENDPOINT",
				Value:   c.GraphQLEndpoint,
				Message: "GraphQL endpoint must be a valid URL",
			})
		}
	}
	
	// Validate Poll Interval
	if c.PollInterval < 100 {
		errors = append(errors, ValidationError{
			Field:   "POLL_INTERVAL",
			Value:   c.PollInterval,
			Message: "poll interval must be at least 100ms to prevent excessive API calls",
		})
	} else if c.PollInterval > 60000 {
		errors = append(errors, ValidationError{
			Field:   "POLL_INTERVAL",
			Value:   c.PollInterval,
			Message: "poll interval should not exceed 60 seconds for real-time updates",
		})
	}
	
	// Validate Network Filter (if provided)
	if c.MainnetOnly {
		// When mainnet only is enabled, we should warn about potential data limitations
		// This is not an error, but could be logged as a warning
	}
	
	if len(errors) > 0 {
		return errors
	}
	
	return nil
}

// ValidateAndWarn validates configuration and logs warnings for non-critical issues
func (c *Config) ValidateAndWarn() error {
	// First run critical validation
	if err := c.Validate(); err != nil {
		return err
	}
	
	// Check for performance warnings
	var warnings []string
	
	if c.PollInterval < 300 {
		warnings = append(warnings, fmt.Sprintf("Poll interval of %dms is quite aggressive and may cause high API load", c.PollInterval))
	}
	
	if c.PollInterval > 5000 {
		warnings = append(warnings, fmt.Sprintf("Poll interval of %dms may result in delayed transfer updates", c.PollInterval))
	}
	
	// Log warnings if any (in a real implementation, you'd use a proper logger)
	for _, warning := range warnings {
		fmt.Printf("[CONFIG WARNING] %s\n", warning)
	}
	
	return nil
}

 