//go:build go1.9
// +build go1.9

package smithytesting

import (
	"strings"
	"testing"
)

func TestEqualXMLUtil(t *testing.T) {
	cases := map[string]struct {
		expectedXML string
		actualXML   string
		expectErr   string
	}{
		"empty": {
			expectedXML: ``,
			actualXML:   ``,
		},
		"emptyWithDiff": {
			expectedXML: ``,
			actualXML:   `<Root></Root>`,
			expectErr:   "XML mismatch",
		},
		"simpleXML": {
			expectedXML: `<Root></Root>`,
			actualXML:   `<Root></Root>`,
		},
		"simpleXMLWithDiff": {
			expectedXML: `<Root></Root>`,
			actualXML:   `<Root>abc</Root>`,
			expectErr:   "XML mismatch",
		},
		"nestedXML": {
			expectedXML: `<Root><abc>123</abc><cde>xyz</cde></Root>`,
			actualXML:   `<Root><abc>123</abc><cde>xyz</cde></Root>`,
		},
		"nestedXMLWithExpectedDiff": {
			expectedXML: `<Root><abc>123</abc><cde>xyz</cde><pqr>234</pqr></Root>`,
			actualXML:   `<Root><abc>123</abc><cde>xyz</cde></Root>`,
			expectErr:   "XML mismatch",
		},
		"nestedXMLWithActualDiff": {
			expectedXML: `<Root><abc>123</abc><cde>xyz</cde></Root>`,
			actualXML:   `<Root><abc>123</abc><cde>xyz</cde><pqr>234</pqr></Root>`,
			expectErr:   "XML mismatch",
		},
		"Array": {
			expectedXML: `<Root><list><member><nested>xyz</nested></member><member><nested>abc</nested></member></list></Root>`,
			actualXML:   `<Root><list><member><nested>xyz</nested></member><member><nested>abc</nested></member></list></Root>`,
		},
		"ArrayWithSecondDiff": {
			expectedXML: `<Root><list><member><nested>xyz</nested></member><member><nested>123</nested></member></list></Root>`,
			actualXML:   `<Root><list><member><nested>xyz</nested></member><member><nested>345</nested></member></list></Root>`,
			expectErr:   "XML mismatch",
		},
		"ArrayWithFirstDiff": {
			expectedXML: `<Root><list><member><nested>abc</nested></member><member><nested>345</nested></member></list></Root>`,
			actualXML:   `<Root><list><member><nested>xyz</nested></member><member><nested>345</nested></member></list></Root>`,
			expectErr:   "XML mismatch",
		},
		"ArrayWithMixedDiff": {
			expectedXML: `<Root><list><member><nested>345</nested></member><member><nested>xyz</nested></member></list></Root>`,
			actualXML:   `<Root><list><member><nested>xyz</nested></member><member><nested>345</nested></member></list></Root>`,
		},
		"ArrayWithRepetitiveMembers": {
			expectedXML: `<Root><list><member><nested>xyz</nested></member><member><nested>xyz</nested></member></list></Root>`,
			actualXML:   `<Root><list><member><nested>xyz</nested></member><member><nested>xyz</nested></member></list></Root>`,
		},
		"Map": {
			expectedXML: `<Root><map><entry><key>abc</key><value>123</value></entry><entry><key>cde</key><value>356</value></entry></map></Root>`,
			actualXML:   `<Root><map><entry><key>abc</key><value>123</value></entry><entry><key>cde</key><value>356</value></entry></map></Root>`,
		},
		"MapWithFirstDiff": {
			expectedXML: `<Root><map><entry><key>bcf</key><value>123</value></entry><entry><key>cde</key><value>356</value></entry></map></Root>`,
			actualXML:   `<Root><map><entry><key>abc</key><value>123</value></entry><entry><key>cde</key><value>356</value></entry></map></Root>`,
			expectErr:   "XML mismatch",
		},
		"MapWithSecondDiff": {
			expectedXML: `<Root><map><entry><key>abc</key><value>123</value></entry><entry><key>cde</key><value>abc</value></entry></map></Root>`,
			actualXML:   `<Root><map><entry><key>abc</key><value>123</value></entry><entry><key>cde</key><value>356</value></entry></map></Root>`,
			expectErr:   "XML mismatch",
		},
		"MapWithMixedDiff": {
			expectedXML: `<Root><map><entry><key>cde</key><value>356</value></entry><entry><key>abc</key><value>123</value></entry></map></Root>`,
			actualXML:   `<Root><map><entry><key>abc</key><value>123</value></entry><entry><key>cde</key><value>356</value></entry></map></Root>`,
		},
		"MismatchCheckforKeyValue": {
			expectedXML: `<Root><map><entry><key>cde</key><value>abc</value></entry><entry><key>abc</key><value>356</value></entry></map></Root>`,
			actualXML:   `<Root><map><entry><key>abc</key><value>123</value></entry><entry><key>cde</key><value>356</value></entry></map></Root>`,
			expectErr:   "XML mismatch",
		},
		"MixMapAndListNestedXML": {
			expectedXML: `<Root><list>mem1</list><list>mem2</list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><alpha><x>gamma</x></alpha></nested></v></map></Root>`,
			actualXML:   `<Root><list>mem1</list><list>mem2</list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><alpha><x>gamma</x></alpha></nested></v></map></Root>`,
		},
		"MixMapAndListNestedXMLWithDiff": {
			expectedXML: `<Root><list>mem1</list><list>mem2</list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><alpha><x>gamma</x></alpha></nested></v></map></Root>`,
			actualXML:   `<Root><list>mem1</list><list>mem2</list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><beta><x>gamma</x></beta></nested></v></map></Root>`,
			expectErr:   "XML mismatch",
		},
		"xmlWithNamespaceAndAttr": {
			expectedXML: `<Root xmlns:ab="https://example.com" attr="apple">value</Root>`,
			actualXML:   `<Root xmlns:ab="https://example.com" attr="apple">value</Root>`,
		},
		"xmlUnorderedAttributes": {
			expectedXML: `<Root atr="banana" attrNew="apple">v</Root>`,
			actualXML:   `<Root attrNew="apple" atr="banana">v</Root>`,
		},
		"xmlAttributesWithDiff": {
			expectedXML: `<Root atr="someAtr" attrNew="apple">v</Root>`,
			actualXML:   `<Root attrNew="apple" atr="banana">v</Root>`,
			expectErr:   "XML mismatch",
		},
		"xmlUnorderedNamespaces": {
			expectedXML: `<Root xmlns:ab="https://example.com" xmlns:baz="https://example2.com">v</Root>`,
			actualXML:   `<Root xmlns:baz="https://example2.com" xmlns:ab="https://example.com">v</Root>`,
		},
		"xmlNamespaceWithDiff": {
			expectedXML: `<Root xmlns:ab="https://example-diff.com" xmlns:baz="https://example2.com">v</Root>`,
			actualXML:   `<Root xmlns:baz="https://example2.com" xmlns:ab="https://example.com">v</Root>`,
			expectErr:   "XML mismatch",
		},
		"NestedWithNamespaceAndAttributes": {
			expectedXML: `<Root xmlns:ab="https://example.com" xmlns:un="https://example2.com" attr="test" attr2="test2"><ab:list>mem1</ab:list><ab:list>mem2</ab:list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><alpha><x>gamma</x></alpha></nested></v></map></Root>`,
			actualXML:   `<Root xmlns:ab="https://example.com" xmlns:un="https://example2.com" attr="test" attr2="test2"><ab:list>mem1</ab:list><ab:list>mem2</ab:list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><alpha><x>gamma</x></alpha></nested></v></map></Root>`,
		},
		"NestedWithNamespaceAndAttributesWithDiff": {
			expectedXML: `<Root xmlns:ab="https://example.com" xmlns:un="https://example2.com" attr="test" attr2="test2"><list>mem2</list><ab:list>mem2</ab:list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><alpha><x>gamma</x></alpha></nested></v></map></Root>`,
			actualXML:   `<Root xmlns:ab="https://example.com" xmlns:un="https://example2.com" attr="test" attr2="test2"><list>mem1</list><ab:list>mem2</ab:list><map><k>abc</k><v><nested><enorm>abc</enorm></nested></v><k>xyz</k><v><nested><alpha><x>gamma</x></alpha></nested></v></map></Root>`,
			expectErr:   "XML mismatch",
		},
		"MalformedXML": {
			expectedXML: `<Root><fmap><key>a</key><key2>a2</key2><value>v</value></fmap><fmap><key>b</key><key2>b2</key2><value>w</value></fmap></Root>`,
			actualXML:   `<Root><fmap><key>a</key><key2>a2</key2><value>v</value></fmap><fmap><key>b</key><key2>b2</key2><value>w</value></fmap></Root>`,
			expectErr:   "malformed xml",
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			actual := []byte(c.actualXML)
			expected := []byte(c.expectedXML)

			err := XMLEqual(actual, expected)
			if err != nil {
				if len(c.expectErr) == 0 {
					t.Fatalf("expected no error while parsing xml, got %v", err)
				} else if !strings.Contains(err.Error(), c.expectErr) {
					t.Fatalf("expected expected XML err to contain %s, got %s", c.expectErr, err.Error())
				}
			} else if len(c.expectErr) != 0 {
				t.Fatalf("expected error %s, got none", c.expectErr)
			}
		})
	}
}
