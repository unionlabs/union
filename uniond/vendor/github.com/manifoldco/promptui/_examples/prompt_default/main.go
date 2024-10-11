package main

import (
	"errors"
	"fmt"
	"os/user"

	"github.com/manifoldco/promptui"
)

func main() {
	validate := func(input string) error {
		if len(input) < 3 {
			return errors.New("Username must have more than 3 characters")
		}
		return nil
	}

	var username string
	u, err := user.Current()
	if err == nil {
		username = u.Username
	}

	prompt := promptui.Prompt{
		Label:    "Username",
		Validate: validate,
		Default:  username,
	}

	result, err := prompt.Run()

	if err != nil {
		fmt.Printf("Prompt failed %v\n", err)
		return
	}

	fmt.Printf("Your username is %q\n", result)
}
