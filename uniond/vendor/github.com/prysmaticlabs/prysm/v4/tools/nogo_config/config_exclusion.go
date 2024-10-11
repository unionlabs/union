package main

// AddExclusion adds an exclusion to the Configs, if they do not exist already.
func (c Configs) AddExclusion(check string, exclusions []string) {
	for _, e := range exclusions {
		if cc, ok := c[check]; !ok {
			c[check] = Config{
				ExcludeFiles: make(map[string]string),
			}
		} else if cc.ExcludeFiles == nil {
			cc.ExcludeFiles = make(map[string]string)
		}
		if _, ok := c[check].ExcludeFiles[e]; !ok {
			c[check].ExcludeFiles[e] = exclusionMessage
		}
	}
}
