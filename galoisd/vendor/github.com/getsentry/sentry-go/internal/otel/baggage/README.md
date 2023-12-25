## Why do we have this "otel/baggage" folder?

The root sentry-go SDK (namely, the Dynamic Sampling functionality) needs an implementation of the [baggage spec](https://www.w3.org/TR/baggage/).
For that reason, we've taken the existing baggage implementation from the [opentelemetry-go](https://github.com/open-telemetry/opentelemetry-go/) repository, and fixed a few things that in our opinion were violating the specification.

These issues are:
1. Baggage string value `one%20two` should be properly parsed as "one two"
1. Baggage string value `one+two` should be parsed as "one+two"
1. Go string value "one two" should be encoded as `one%20two` (percent encoding), and NOT as `one+two` (URL query encoding).
1. Go string value "1=1" might be encoded as `1=1`, because the spec says: "Note, value MAY contain any number of the equal sign (=) characters. Parsers MUST NOT assume that the equal sign is only used to separate key and value.". `1%3D1` is also valid, but to simplify the implementation we're not doing it.

Changes were made in this PR: https://github.com/getsentry/sentry-go/pull/568
