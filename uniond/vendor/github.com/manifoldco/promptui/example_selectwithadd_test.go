package promptui

import "fmt"

// This example shows how to create a SelectWithAdd that will add each new item it is given to the
// list of items until one is chosen.
func ExampleSelectWithAdd() {
	items := []string{"Vim", "Emacs", "Sublime", "VSCode", "Atom"}
	index := -1
	var result string
	var err error

	for index < 0 {
		prompt := SelectWithAdd{
			Label:    "What's your text editor",
			Items:    items,
			AddLabel: "Add your own",
		}

		index, result, err = prompt.Run()

		if index == -1 {
			items = append(items, result)
		}
	}

	if err != nil {
		fmt.Printf("Prompt failed %v\n", err)
		return
	}

	fmt.Printf("You choose %s\n", result)
}
