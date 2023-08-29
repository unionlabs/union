{
	checks: (.checks // {}), 
	packages: (.packages // {}), 
	devShells: (.devShells // {})
} |
to_entries |
map(.key as $top_attr | 
	.value | 
	to_entries | 
	map(.key as $sys | 
		.value |
		to_entries |
		map(.key as $attr | 
			{
				name: "",
				description: "",
				top_attr: $top_attr,
				system: $sys,
				attr: $attr
			} + .value
		)
	)
) | 
flatten
