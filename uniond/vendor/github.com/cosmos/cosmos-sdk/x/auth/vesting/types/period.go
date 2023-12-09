package types

import (
	"fmt"
	"strings"
	"time"

	sdk "github.com/cosmos/cosmos-sdk/types"
)

// Periods stores all vesting periods passed as part of a PeriodicVestingAccount
type Periods []Period

// Duration is converts the period Length from seconds to a time.Duration
func (p Period) Duration() time.Duration {
	return time.Duration(p.Length) * time.Second
}

// TotalLength return the total length in seconds for a period
func (p Periods) TotalLength() int64 {
	var total int64
	for _, period := range p {
		total += period.Length
	}
	return total
}

// TotalDuration returns the total duration of the period
func (p Periods) TotalDuration() time.Duration {
	len := p.TotalLength()
	return time.Duration(len) * time.Second
}

// TotalDuration returns the sum of coins for the period
func (p Periods) TotalAmount() sdk.Coins {
	total := sdk.Coins{}
	for _, period := range p {
		total = total.Add(period.Amount...)
	}
	return total
}

// String implements the fmt.Stringer interface
func (p Periods) String() string {
	periodsListString := make([]string, len(p))
	for _, period := range p {
		periodsListString = append(periodsListString, period.String())
	}

	return strings.TrimSpace(fmt.Sprintf(`Vesting Periods:
		%s`, strings.Join(periodsListString, ", ")))
}
