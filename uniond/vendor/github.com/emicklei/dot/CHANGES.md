# Change history of the dot package


## v1.6.1 - 2024-01-17

- deprecated MermaidShapeCirle => use MermaidShapeCircle

## v1.6.0 - 2023-07-30

- add Record builder

## v1.5.0 - 2023-06-20

- add GetID on Graph

## v1.4.0 - 2023-03-28

- add Composite extension

## v1.3.0 - 2023-02-21

- Escape/quote mermaid
- add FindNodeWithLabel 

## v1.2.0 - 2022-11-18

- add BidirectionalEdge #28

## v1.1.0 - 2022-11-07

- add support for Mermaid graph out.

## v1.0.0 - 2022-06-22

- add support for port, see https://github.com/emicklei/dot/pull/25 (thx v-electrolux)

## v0.16.0 - 2021-05-04

- add DeleteNote, see https://github.com/emicklei/dot/pull/24

## v0.15.0 - 2020-10-30

- add Node initializer, see Issue #15
- add Edge initializer

## v0.14.0 - 2020-08-25

- add Attrs for conveniently adding multiple label=value attribute pairs.

## v0.13.0 - 2020-08-22

- add FindSubgraph

## v0.12.0 - 2020-08-20

- Added style methods to Edge to easily add bold,dotted and dashed lines. (#21)

## v0.11.0 - 2020-05-16

- add functionality to find node by id
- add function to find all nodes of a graph

## v0.10.2 - 2020-01-31 

- Fix indexing subgraphs by label ; must use id. Issue #16
- Add Label(newLabel) to Graph
- Add Delete(key) to AttributesMap
- Use internal ids for subgraphs

## v0.10.0

- Allow setting same rank for a group of nodes
- Introduce Literal attribute type
- Introduce Node.Label(string) function

## v0.9.2 and earlier

- Add support for HTML attributes.
- fixed undirected transitions
- Change how node is printed, so that attributes only affect individual node
