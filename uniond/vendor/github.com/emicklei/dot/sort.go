package dot

import "sort"

func (g *Graph) sortedNodesKeys() (keys []string) {
	for each := range g.nodes {
		keys = append(keys, each)
	}
	sort.StringSlice(keys).Sort()
	return
}
func (g *Graph) sortedEdgesFromKeys() (keys []string) {
	for each := range g.edgesFrom {
		keys = append(keys, each)
	}
	sort.StringSlice(keys).Sort()
	return
}
func (g *Graph) sortedSubgraphsKeys() (keys []string) {
	for each := range g.subgraphs {
		keys = append(keys, each)
	}
	sort.StringSlice(keys).Sort()
	return
}
