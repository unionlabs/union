package test_vector_utils

/*
var hashCache = make(map[string]HashMap)

func GetHash(path string) (HashMap, error) {
	path, err := filepath.Abs(path)
	if err != nil {
		return nil, err
	}
	if h, ok := hashCache[path]; ok {
		return h, nil
	}
	var bytes []byte
	if bytes, err = os.ReadFile(path); err == nil {
		var asMap map[string]interface{}
		if err = json.Unmarshal(bytes, &asMap); err != nil {
			return nil, err
		}

		res := make(HashMap, 0, len(asMap))

		for k, v := range asMap {
			var entry RationalTriplet
			if _, err = entry.value.SetInterface(v); err != nil {
				return nil, err
			}

			key := strings.Split(k, ",")

			switch len(key) {
			case 1:
				entry.key2Present = false
			case 2:
				entry.key2Present = true
				if _, err = entry.key2.SetInterface(key[1]); err != nil {
					return nil, err
				}
			default:
				return nil, fmt.Errorf("cannot parse %T as one or two field elements", v)
			}
			if _, err = entry.key1.SetInterface(key[0]); err != nil {
				return nil, err
			}

			res = append(res, &entry)
		}

		res.sort()

		hashCache[path] = res

		return res, nil

	} else {
		return nil, err
	}
}

func (m *HashMap) SaveUsedEntries(path string) error {

	var sb strings.Builder
	sb.WriteRune('[')

	first := true

	for _, element := range *m {
		if !element.used {
			continue
		}
		if !first {
			sb.WriteRune(',')
		}
		first = false
		sb.WriteString("\n\t")
		element.WriteKeyValue(&sb)
	}

	if !first {
		sb.WriteRune(',')
	}

	sb.WriteString("\n]")

	return os.WriteFile(path, []byte(sb.String()), 0)
}

type HashMap []*RationalTriplet

type RationalTriplet struct {
	key1        small_rational.SmallRational
	key2        small_rational.SmallRational
	key2Present bool
	value       small_rational.SmallRational
	used        bool
}

func (t *RationalTriplet) WriteKeyValue(sb *strings.Builder) {
	sb.WriteString("\t\"")
	sb.WriteString(t.key1.String())
	if t.key2Present {
		sb.WriteRune(',')
		sb.WriteString(t.key2.String())
	}
	sb.WriteString("\":")
	if valueBytes, err := json.Marshal(ElementToInterface(&t.value)); err == nil {
		sb.WriteString(string(valueBytes))
	} else {
		panic(err.Error())
	}
}

func (m *HashMap) sort() {
	sort.Slice(*m, func(i, j int) bool {
		return (*m)[i].CmpKey((*m)[j]) <= 0
	})
}

func (m *HashMap) find(toFind *RationalTriplet) small_rational.SmallRational {
	i := sort.Search(len(*m), func(i int) bool { return (*m)[i].CmpKey(toFind) >= 0 })

	if i < len(*m) && (*m)[i].CmpKey(toFind) == 0 {
		(*m)[i].used = true
		return (*m)[i].value
	}

	// if not found, add it:
	if _, err := toFind.value.SetInterface(rand.Int63n(11) - 5); err != nil {
		panic(err.Error())
	}
	toFind.used = true
	*m = append(*m, toFind)
	m.sort() //Inefficient, but it's okay. This is only run when a new test case is introduced

	return toFind.value
}

func (t *RationalTriplet) CmpKey(o *RationalTriplet) int {
	if cmp1 := t.key1.Cmp(&o.key1); cmp1 != 0 {
		return cmp1
	}

	if t.key2Present {
		if o.key2Present {
			return t.key2.Cmp(&o.key2)
		}
		return 1
	} else {
		if o.key2Present {
			return -1
		}
		return 0
	}
}

type MapHashTranscript struct {
	HashMap         HashMap
	stateValid      bool
	resultAvailable bool
	state           small_rational.SmallRational
}

func (m *HashMap) Hash(x *small_rational.SmallRational, y *small_rational.SmallRational) small_rational.SmallRational {

	toFind := RationalTriplet{
		key1:        *x,
		key2Present: y != nil,
	}

	if y != nil {
		toFind.key2 = *y
	}

	return m.find(&toFind)
}

func (m *MapHashTranscript) Update(i ...interface{}) {
	if len(i) > 0 {
		for _, x := range i {

			var xElement small_rational.SmallRational
			if _, err := xElement.SetInterface(x); err != nil {
				panic(err.Error())
			}
			if m.stateValid {
				m.state = m.HashMap.Hash(&xElement, &m.state)
			} else {
				m.state = m.HashMap.Hash(&xElement, nil)
			}

			m.stateValid = true
		}
	} else { //just hash the state itself
		if !m.stateValid {
			panic("nothing to hash")
		}
		m.state = m.HashMap.Hash(&m.state, nil)
	}
	m.resultAvailable = true
}

func (m *MapHashTranscript) Next(i ...interface{}) small_rational.SmallRational {

	if len(i) > 0 || !m.resultAvailable {
		m.Update(i...)
	}
	m.resultAvailable = false
	return m.state
}

func (m *MapHashTranscript) NextN(N int, i ...interface{}) []small_rational.SmallRational {

	if len(i) > 0 {
		m.Update(i...)
	}

	res := make([]small_rational.SmallRational, N)

	for n := range res {
		res[n] = m.Next()
	}

	return res
}

func SliceToElementSlice(slice []interface{}) ([]small_rational.SmallRational, error) {
	elementSlice := make([]small_rational.SmallRational, len(slice))
	for i, v := range slice {
		if _, err := elementSlice[i].SetInterface(v); err != nil {
			return nil, err
		}
	}
	return elementSlice, nil
}

func SliceEquals(a []small_rational.SmallRational, b []small_rational.SmallRational) error {
	if len(a) != len(b) {
		return fmt.Errorf("length mismatch %d≠%d", len(a), len(b))
	}
	for i := range a {
		if !a[i].Equal(&b[i]) {
			return fmt.Errorf("at index %d: %s ≠ %s", i, a[i].String(), b[i].String())
		}
	}
	return nil
}

*/
