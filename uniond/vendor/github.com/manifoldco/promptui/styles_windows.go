package promptui

// These are the default icons used bu promptui for select and prompts. They can either be overridden directly
// from these variable or customized through the use of custom templates
var (
	// IconInitial is the icon used when starting in prompt mode and the icon next to the label when
	// starting in select mode.
	IconInitial = Styler(FGBlue)("?")

	// IconGood is the icon used when a good answer is entered in prompt mode.
	IconGood = Styler(FGGreen)("v")

	// IconWarn is the icon used when a good, but potentially invalid answer is entered in prompt mode.
	IconWarn = Styler(FGYellow)("!")

	// IconBad is the icon used when a bad answer is entered in prompt mode.
	IconBad = Styler(FGRed)("x")

	// IconSelect is the icon used to identify the currently selected item in select mode.
	IconSelect = Styler(FGBold)(">")
)
