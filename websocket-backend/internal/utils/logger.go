package utils

import (
	"encoding/json"
	"fmt"
	"log"
	"os"
	"runtime"
	"strings"
	"time"
)

// LogLevel represents the severity level of a log entry
type LogLevel string

const (
	DEBUG LogLevel = "DEBUG"
	INFO  LogLevel = "INFO"
	WARN  LogLevel = "WARN"
	ERROR LogLevel = "ERROR"
	FATAL LogLevel = "FATAL"
)

// LogEntry represents a structured log entry
type LogEntry struct {
	Timestamp string                 `json:"timestamp"`
	Level     LogLevel               `json:"level"`
	Component string                 `json:"component"`
	Message   string                 `json:"message"`
	Fields    map[string]interface{} `json:"fields,omitempty"`
	Caller    string                 `json:"caller,omitempty"`
}

// Logger provides structured logging functionality
type Logger struct {
	component   string
	level       LogLevel
	jsonFormat  bool
	includeCaller bool
}

// NewLogger creates a new structured logger for a component
func NewLogger(component string) *Logger {
	return &Logger{
		component:     component,
		level:         INFO, // Default level
		jsonFormat:    true, // JSON format for production
		includeCaller: false, // Disable caller info by default for performance
	}
}

// SetLevel sets the minimum log level
func (l *Logger) SetLevel(level LogLevel) {
	l.level = level
}

// SetFormat sets the output format (true for JSON, false for human-readable)
func (l *Logger) SetFormat(jsonFormat bool) {
	l.jsonFormat = jsonFormat
}

// SetIncludeCaller enables/disables caller information in logs
func (l *Logger) SetIncludeCaller(include bool) {
	l.includeCaller = include
}

// shouldLog checks if a message should be logged based on the current level
func (l *Logger) shouldLog(level LogLevel) bool {
	levels := map[LogLevel]int{
		DEBUG: 0,
		INFO:  1,
		WARN:  2,
		ERROR: 3,
		FATAL: 4,
	}
	
	return levels[level] >= levels[l.level]
}

// getCaller returns the caller information
func (l *Logger) getCaller() string {
	if !l.includeCaller {
		return ""
	}
	
	_, file, line, ok := runtime.Caller(3) // Skip log method, public method, and this method
	if !ok {
		return "unknown"
	}
	
	// Get just the filename, not the full path
	parts := strings.Split(file, "/")
	filename := parts[len(parts)-1]
	
	return fmt.Sprintf("%s:%d", filename, line)
}

// log writes a log entry
func (l *Logger) log(level LogLevel, message string, fields map[string]interface{}) {
	if !l.shouldLog(level) {
		return
	}
	
	entry := LogEntry{
		Timestamp: time.Now().UTC().Format(time.RFC3339),
		Level:     level,
		Component: l.component,
		Message:   message,
		Fields:    fields,
		Caller:    l.getCaller(),
	}
	
	if l.jsonFormat {
		jsonData, err := json.Marshal(entry)
		if err != nil {
			// Fallback to simple logging if JSON marshaling fails
			log.Printf("[%s] %s: %s", level, l.component, message)
			return
		}
		fmt.Println(string(jsonData))
	} else {
		// Human-readable format
		fieldsStr := ""
		if len(fields) > 0 {
			var fieldPairs []string
			for k, v := range fields {
				fieldPairs = append(fieldPairs, fmt.Sprintf("%s=%v", k, v))
			}
			fieldsStr = " " + strings.Join(fieldPairs, " ")
		}
		
		caller := ""
		if entry.Caller != "" {
			caller = fmt.Sprintf(" [%s]", entry.Caller)
		}
		
		fmt.Printf("[%s] %s: %s%s%s\n", level, l.component, message, fieldsStr, caller)
	}
}

// Debug logs a debug message
func (l *Logger) Debug(message string, fields ...map[string]interface{}) {
	var f map[string]interface{}
	if len(fields) > 0 {
		f = fields[0]
	}
	l.log(DEBUG, message, f)
}

// Info logs an info message
func (l *Logger) Info(message string, fields ...map[string]interface{}) {
	var f map[string]interface{}
	if len(fields) > 0 {
		f = fields[0]
	}
	l.log(INFO, message, f)
}

// Warn logs a warning message
func (l *Logger) Warn(message string, fields ...map[string]interface{}) {
	var f map[string]interface{}
	if len(fields) > 0 {
		f = fields[0]
	}
	l.log(WARN, message, f)
}

// Error logs an error message
func (l *Logger) Error(message string, fields ...map[string]interface{}) {
	var f map[string]interface{}
	if len(fields) > 0 {
		f = fields[0]
	}
	l.log(ERROR, message, f)
}

// Fatal logs a fatal message and exits
func (l *Logger) Fatal(message string, fields ...map[string]interface{}) {
	var f map[string]interface{}
	if len(fields) > 0 {
		f = fields[0]
	}
	l.log(FATAL, message, f)
	os.Exit(1)
}

// WithFields returns a new logger with additional fields
func (l *Logger) WithFields(fields map[string]interface{}) *FieldLogger {
	return &FieldLogger{
		logger: l,
		fields: fields,
	}
}

// FieldLogger is a logger with pre-set fields
type FieldLogger struct {
	logger *Logger
	fields map[string]interface{}
}

// Debug logs a debug message with pre-set fields
func (fl *FieldLogger) Debug(message string, additionalFields ...map[string]interface{}) {
	fields := make(map[string]interface{})
	for k, v := range fl.fields {
		fields[k] = v
	}
	if len(additionalFields) > 0 {
		for k, v := range additionalFields[0] {
			fields[k] = v
		}
	}
	fl.logger.log(DEBUG, message, fields)
}

// Info logs an info message with pre-set fields
func (fl *FieldLogger) Info(message string, additionalFields ...map[string]interface{}) {
	fields := make(map[string]interface{})
	for k, v := range fl.fields {
		fields[k] = v
	}
	if len(additionalFields) > 0 {
		for k, v := range additionalFields[0] {
			fields[k] = v
		}
	}
	fl.logger.log(INFO, message, fields)
}

// Warn logs a warning message with pre-set fields
func (fl *FieldLogger) Warn(message string, additionalFields ...map[string]interface{}) {
	fields := make(map[string]interface{})
	for k, v := range fl.fields {
		fields[k] = v
	}
	if len(additionalFields) > 0 {
		for k, v := range additionalFields[0] {
			fields[k] = v
		}
	}
	fl.logger.log(WARN, message, fields)
}

// Error logs an error message with pre-set fields
func (fl *FieldLogger) Error(message string, additionalFields ...map[string]interface{}) {
	fields := make(map[string]interface{})
	for k, v := range fl.fields {
		fields[k] = v
	}
	if len(additionalFields) > 0 {
		for k, v := range additionalFields[0] {
			fields[k] = v
		}
	}
	fl.logger.log(ERROR, message, fields)
}

// Global logger instances for different components
var (
	ServerLogger    = NewLogger("SERVER")
	WSLogger        = NewLogger("WS")
	GraphQLLogger   = NewLogger("GRAPHQL")
	StatsLogger     = NewLogger("STATS")
	BroadcastLogger = NewLogger("BROADCAST")
	PollLogger      = NewLogger("POLL")
	ScheduleLogger  = NewLogger("SCHEDULE")
	ChartLogger     = NewLogger("CHART")
	EnhanceLogger   = NewLogger("ENHANCE")
	StreamLogger    = NewLogger("STREAM")
	ChainsLogger    = NewLogger("CHAINS")
)

// InitializeLogging sets up logging configuration based on environment
func InitializeLogging(isDevelopment bool) {
	if isDevelopment {
		// Development: human-readable format, debug level
		loggers := []*Logger{
			ServerLogger, WSLogger, GraphQLLogger, StatsLogger,
			BroadcastLogger, PollLogger, ScheduleLogger, ChartLogger,
			EnhanceLogger, StreamLogger, ChainsLogger,
		}
		
		for _, logger := range loggers {
			logger.SetFormat(false) // Human-readable
			logger.SetLevel(DEBUG)  // Show all logs
			logger.SetIncludeCaller(true) // Include caller info for debugging
		}
	} else {
		// Production: JSON format, info level
		loggers := []*Logger{
			ServerLogger, WSLogger, GraphQLLogger, StatsLogger,
			BroadcastLogger, PollLogger, ScheduleLogger, ChartLogger,
			EnhanceLogger, StreamLogger, ChainsLogger,
		}
		
		for _, logger := range loggers {
			logger.SetFormat(true)  // JSON format
			logger.SetLevel(INFO)   // Info and above only
			logger.SetIncludeCaller(false) // No caller info for performance
		}
	}
} 