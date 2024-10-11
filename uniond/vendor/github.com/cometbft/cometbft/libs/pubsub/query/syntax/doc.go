// Package syntax defines a scanner and parser for the CometBFT event filter
// query language. A query selects events by their types and attribute values.
//
// # Grammar
//
// The grammar of the query language is defined by the following EBNF:
//
//	query      = conditions EOF
//	conditions = condition {"AND" condition}
//	condition  = tag comparison
//	comparison = equal / order / contains / "EXISTS"
//	equal      = "=" (date / number / time / value)
//	order      = cmp (date / number / time)
//	contains   = "CONTAINS" value
//	cmp        = "<" / "<=" / ">" / ">="
//
// The lexical terms are defined here using RE2 regular expression notation:
//
//	// The name of an event attribute (type.value)
//	 tag   = #`^[\w]+[\.-\w]?$`
//
//	// A datestamp (YYYY-MM-DD)
//	date   = #'DATE \d{4}-\d{2}-\d{2}'
//
//	// A number with optional fractional parts (0, 10, 3.25)
//	number = #'\d+(\.\d+)?'
//
//	// An RFC3339 timestamp (2021-11-23T22:04:19-09:00)
//	time   = #'TIME \d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}([-+]\d{2}:\d{2}|Z)'
//
//	// A quoted literal string value ('a b c')
//	value  = #'\'[^\']*\''
package syntax
