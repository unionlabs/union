package promptui

import (
	"fmt"
	"strconv"
)

// This example shows how to use the prompt validator and templates to create a stylized prompt.
// The validator will make sure the value entered is a parseable float while the templates will
// color the value to show validity.
func ExamplePrompt() {
	// The validate function follows the required validator signature.
	validate := func(input string) error {
		_, err := strconv.ParseFloat(input, 64)
		return err
	}

	// Each template displays the data received from the prompt with some formatting.
	templates := &PromptTemplates{
		Prompt:  "{{ . }} ",
		Valid:   "{{ . | green }} ",
		Invalid: "{{ . | red }} ",
		Success: "{{ . | bold }} ",
	}

	prompt := Prompt{
		Label:     "Spicy Level",
		Templates: templates,
		Validate:  validate,
	}

	result, err := prompt.Run()

	if err != nil {
		fmt.Printf("Prompt failed %v\n", err)
		return
	}

	// The result of the prompt, if valid, is displayed in a formatted message.
	fmt.Printf("You answered %s\n", result)
}
