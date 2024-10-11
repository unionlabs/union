package promptui

import (
	"errors"
	"fmt"
	"strconv"
)

// This is an example for the Prompt mode of promptui. In this example, a prompt is created
// with a validator function that validates the given value to make sure its a number.
// If successful, it will output the chosen number in a formatted message.
func Example_prompt() {
	validate := func(input string) error {
		_, err := strconv.ParseFloat(input, 64)
		if err != nil {
			return errors.New("Invalid number")
		}
		return nil
	}

	prompt := Prompt{
		Label:    "Number",
		Validate: validate,
	}

	result, err := prompt.Run()

	if err != nil {
		fmt.Printf("Prompt failed %v\n", err)
		return
	}

	fmt.Printf("You choose %q\n", result)
}

// This is an example for the Select mode of promptui. In this example, a select is created with
// the days of the week as its items. When an item is selected, the selected day will be displayed
// in a formatted message.
func Example_select() {
	prompt := Select{
		Label: "Select Day",
		Items: []string{"Monday", "Tuesday", "Wednesday", "Thursday", "Friday",
			"Saturday", "Sunday"},
	}

	_, result, err := prompt.Run()

	if err != nil {
		fmt.Printf("Prompt failed %v\n", err)
		return
	}

	fmt.Printf("You choose %q\n", result)
}
