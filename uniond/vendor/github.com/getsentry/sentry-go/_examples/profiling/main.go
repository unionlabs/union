//	go run main.go
//
// To actually report events to Sentry, set the DSN either by editing the
// appropriate line below or setting the environment variable SENTRY_DSN to
// match the DSN of your Sentry project.
package main

import (
	"context"
	"fmt"
	"log"
	"runtime"
	"sync"
	"time"

	"github.com/getsentry/sentry-go"
)

func main() {
	err := sentry.Init(sentry.ClientOptions{
		// Either set your DSN here or set the SENTRY_DSN environment variable.
		Dsn: "",
		// Enable printing of SDK debug messages.
		// Useful when getting started or trying to figure something out.
		Debug:              true,
		EnableTracing:      true,
		TracesSampleRate:   1.0,
		ProfilesSampleRate: 1.0,
	})

	// Flush buffered events before the program terminates.
	// Set the timeout to the maximum duration the program can afford to wait.
	defer sentry.Flush(2 * time.Second)

	if err != nil {
		log.Fatalf("sentry.Init: %s", err)
	}
	ctx := context.Background()
	tx := sentry.StartTransaction(ctx, "top")

	fmt.Println("Finding prime numbers")
	var wg sync.WaitGroup
	wg.Add(10)
	for i := 0; i < 10; i++ {
		go func(num int) {
			defer wg.Done()
			span := tx.StartChild(fmt.Sprintf("Goroutine %d", num))
			defer span.Finish()
			for i := 0; i < num; i++ {
				_ = findPrimeNumber(50000)
				runtime.Gosched() // we need to manually yield this busy loop
			}
			fmt.Printf("routine %d done\n", num)
		}(i)
	}
	wg.Wait()
	fmt.Println("all")
	tx.Finish()
}

func findPrimeNumber(n int) int {
	count := 0
	a := 2
	for count < n {
		b := 2
		prime := true // to check if found a prime
		for b*b <= a {
			if a%b == 0 {
				prime = false
				break
			}
			b++
		}
		if prime {
			count++
		}
		a++
	}
	return a - 1
}
