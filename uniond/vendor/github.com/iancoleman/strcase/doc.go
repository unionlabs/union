// Package strcase converts strings to various cases. See the conversion table below:
//   | Function                        | Result             |
//   |---------------------------------|--------------------|
//   | ToSnake(s)                      | any_kind_of_string |
//   | ToScreamingSnake(s)             | ANY_KIND_OF_STRING |
//   | ToKebab(s)                      | any-kind-of-string |
//   | ToScreamingKebab(s)             | ANY-KIND-OF-STRING |
//   | ToDelimited(s, '.')             | any.kind.of.string |
//   | ToScreamingDelimited(s, '.')    | ANY.KIND.OF.STRING |
//   | ToCamel(s)                      | AnyKindOfString    |
//   | ToLowerCamel(s)                 | anyKindOfString    |
package strcase
