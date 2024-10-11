package main

import (
	"fmt"
	"math/rand"
	"time"

	"github.com/chzyer/readline"
)
import "log"

func main() {
	rl, err := readline.NewEx(&readline.Config{
		UniqueEditLine: true,
	})
	if err != nil {
		panic(err)
	}
	defer rl.Close()

	rl.SetPrompt("username: ")
	username, err := rl.Readline()
	if err != nil {
		return
	}
	rl.ResetHistory()
	log.SetOutput(rl.Stderr())

	fmt.Fprintln(rl, "Hi,", username+"! My name is Dave.")
	rl.SetPrompt(username + "> ")

	done := make(chan struct{})
	go func() {
		rand.Seed(time.Now().Unix())
	loop:
		for {
			select {
			case <-time.After(time.Duration(rand.Intn(20)) * 100 * time.Millisecond):
			case <-done:
				break loop
			}
			log.Println("Dave:", "hello")
		}
		log.Println("Dave:", "bye")
		done <- struct{}{}
	}()

	for {
		ln := rl.Line()
		if ln.CanContinue() {
			continue
		} else if ln.CanBreak() {
			break
		}
		log.Println(username+":", ln.Line)
	}
	rl.Clean()
	done <- struct{}{}
	<-done
}
