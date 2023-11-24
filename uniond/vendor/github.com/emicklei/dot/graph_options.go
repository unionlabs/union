package dot

type GraphOption interface {
	Apply(*Graph)
}

type ClusterOption struct{}

func (o ClusterOption) Apply(g *Graph) {
	g.beCluster()
}

var (
	Strict     = GraphTypeOption{"strict"}
	Undirected = GraphTypeOption{"graph"}
	Directed   = GraphTypeOption{"digraph"}
	Sub        = GraphTypeOption{"subgraph"}
)

type GraphTypeOption struct {
	Name string
}

func (o GraphTypeOption) Apply(g *Graph) {
	g.graphType = o.Name
}
