package dot

type GraphOption interface {
	Apply(*Graph)
}

type ClusterOption struct{}

func (o ClusterOption) Apply(g *Graph) {
	g.beCluster()
}

var (
	Strict     = GraphTypeOption{"strict"} // only for graph and digraph, not for subgraph
	Undirected = GraphTypeOption{"graph"}
	Directed   = GraphTypeOption{"digraph"}
	Sub        = GraphTypeOption{"subgraph"}
)

type GraphTypeOption struct {
	Name string
}

func (o GraphTypeOption) Apply(g *Graph) {
	if o.Name == Strict.Name {
		g.isStrict = true
		return
	}
	g.graphType = o.Name
}
