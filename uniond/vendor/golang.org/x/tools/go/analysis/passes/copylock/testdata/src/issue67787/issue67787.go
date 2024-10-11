package issue67787

import "sync"

type T struct{ mu sync.Mutex }
type T1 struct{ t *T }

func NewT1() *T1 { return &T1{T} } // no analyzer diagnostic about T
