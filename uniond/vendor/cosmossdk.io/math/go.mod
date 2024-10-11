module cosmossdk.io/math

go 1.20

require (
	github.com/stretchr/testify v1.8.4
	golang.org/x/exp v0.0.0-20221205204356-47842c84f3db
	sigs.k8s.io/yaml v1.4.0
)

require (
	github.com/davecgh/go-spew v1.1.1 // indirect
	github.com/kr/pretty v0.3.1 // indirect
	github.com/pmezard/go-difflib v1.0.0 // indirect
	gopkg.in/check.v1 v1.0.0-20201130134442-10cb98267c6c // indirect
	gopkg.in/yaml.v3 v3.0.1 // indirect
)

// Issue with math.Int{}.Size() implementation.
retract [v1.1.0, v1.1.1]
