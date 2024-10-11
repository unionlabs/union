// Copyright 2019 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package metamorphic

import (
	"fmt"
	"go/scanner"
	"go/token"
	"reflect"
	"strconv"
	"strings"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
)

type methodInfo struct {
	constructor func() op
	validTags   uint32
}

func makeMethod(i interface{}, tags ...objTag) *methodInfo {
	var validTags uint32
	for _, tag := range tags {
		validTags |= 1 << tag
	}

	t := reflect.TypeOf(i)
	return &methodInfo{
		constructor: func() op {
			return reflect.New(t).Interface().(op)
		},
		validTags: validTags,
	}
}

// args returns the receiverID, targetID and arguments for the op. The
// receiverID is the ID of the object the op will be applied to. The targetID
// is the ID of the object for assignment. If the method does not return a new
// object, then targetID will be nil. The argument list is just what it sounds
// like: the list of arguments for the operation.
func opArgs(op op) (receiverID *objID, targetID *objID, args []interface{}) {
	switch t := op.(type) {
	case *applyOp:
		return &t.writerID, nil, []interface{}{&t.batchID}
	case *checkpointOp:
		return nil, nil, []interface{}{&t.spans}
	case *closeOp:
		return &t.objID, nil, nil
	case *compactOp:
		return nil, nil, []interface{}{&t.start, &t.end, &t.parallelize}
	case *batchCommitOp:
		return &t.batchID, nil, nil
	case *dbRatchetFormatMajorVersionOp:
		return nil, nil, []interface{}{&t.vers}
	case *dbRestartOp:
		return nil, nil, nil
	case *deleteOp:
		return &t.writerID, nil, []interface{}{&t.key}
	case *deleteRangeOp:
		return &t.writerID, nil, []interface{}{&t.start, &t.end}
	case *iterFirstOp:
		return &t.iterID, nil, nil
	case *flushOp:
		return nil, nil, nil
	case *getOp:
		return &t.readerID, nil, []interface{}{&t.key}
	case *ingestOp:
		return nil, nil, []interface{}{&t.batchIDs}
	case *initOp:
		return nil, nil, []interface{}{&t.batchSlots, &t.iterSlots, &t.snapshotSlots}
	case *iterLastOp:
		return &t.iterID, nil, nil
	case *mergeOp:
		return &t.writerID, nil, []interface{}{&t.key, &t.value}
	case *newBatchOp:
		return nil, &t.batchID, nil
	case *newIndexedBatchOp:
		return nil, &t.batchID, nil
	case *newIterOp:
		return &t.readerID, &t.iterID, []interface{}{&t.lower, &t.upper, &t.keyTypes, &t.filterMin, &t.filterMax, &t.useL6Filters, &t.maskSuffix}
	case *newIterUsingCloneOp:
		return &t.existingIterID, &t.iterID, []interface{}{&t.refreshBatch, &t.lower, &t.upper, &t.keyTypes, &t.filterMin, &t.filterMax, &t.useL6Filters, &t.maskSuffix}
	case *newSnapshotOp:
		return nil, &t.snapID, []interface{}{&t.bounds}
	case *iterNextOp:
		return &t.iterID, nil, []interface{}{&t.limit}
	case *iterNextPrefixOp:
		return &t.iterID, nil, nil
	case *iterCanSingleDelOp:
		return &t.iterID, nil, []interface{}{}
	case *iterPrevOp:
		return &t.iterID, nil, []interface{}{&t.limit}
	case *iterSeekLTOp:
		return &t.iterID, nil, []interface{}{&t.key, &t.limit}
	case *iterSeekGEOp:
		return &t.iterID, nil, []interface{}{&t.key, &t.limit}
	case *iterSeekPrefixGEOp:
		return &t.iterID, nil, []interface{}{&t.key}
	case *setOp:
		return &t.writerID, nil, []interface{}{&t.key, &t.value}
	case *iterSetBoundsOp:
		return &t.iterID, nil, []interface{}{&t.lower, &t.upper}
	case *iterSetOptionsOp:
		return &t.iterID, nil, []interface{}{&t.lower, &t.upper, &t.keyTypes, &t.filterMin, &t.filterMax, &t.useL6Filters, &t.maskSuffix}
	case *singleDeleteOp:
		return &t.writerID, nil, []interface{}{&t.key, &t.maybeReplaceDelete}
	case *rangeKeyDeleteOp:
		return &t.writerID, nil, []interface{}{&t.start, &t.end}
	case *rangeKeySetOp:
		return &t.writerID, nil, []interface{}{&t.start, &t.end, &t.suffix, &t.value}
	case *rangeKeyUnsetOp:
		return &t.writerID, nil, []interface{}{&t.start, &t.end, &t.suffix}
	}
	panic(fmt.Sprintf("unsupported op type: %T", op))
}

var methods = map[string]*methodInfo{
	"Apply":                     makeMethod(applyOp{}, dbTag, batchTag),
	"Checkpoint":                makeMethod(checkpointOp{}, dbTag),
	"Clone":                     makeMethod(newIterUsingCloneOp{}, iterTag),
	"Close":                     makeMethod(closeOp{}, dbTag, batchTag, iterTag, snapTag),
	"Commit":                    makeMethod(batchCommitOp{}, batchTag),
	"Compact":                   makeMethod(compactOp{}, dbTag),
	"Delete":                    makeMethod(deleteOp{}, dbTag, batchTag),
	"DeleteRange":               makeMethod(deleteRangeOp{}, dbTag, batchTag),
	"First":                     makeMethod(iterFirstOp{}, iterTag),
	"Flush":                     makeMethod(flushOp{}, dbTag),
	"Get":                       makeMethod(getOp{}, dbTag, batchTag, snapTag),
	"Ingest":                    makeMethod(ingestOp{}, dbTag),
	"Init":                      makeMethod(initOp{}, dbTag),
	"Last":                      makeMethod(iterLastOp{}, iterTag),
	"Merge":                     makeMethod(mergeOp{}, dbTag, batchTag),
	"NewBatch":                  makeMethod(newBatchOp{}, dbTag),
	"NewIndexedBatch":           makeMethod(newIndexedBatchOp{}, dbTag),
	"NewIter":                   makeMethod(newIterOp{}, dbTag, batchTag, snapTag),
	"NewSnapshot":               makeMethod(newSnapshotOp{}, dbTag),
	"Next":                      makeMethod(iterNextOp{}, iterTag),
	"NextPrefix":                makeMethod(iterNextPrefixOp{}, iterTag),
	"InternalNext":              makeMethod(iterCanSingleDelOp{}, iterTag),
	"Prev":                      makeMethod(iterPrevOp{}, iterTag),
	"RangeKeyDelete":            makeMethod(rangeKeyDeleteOp{}, dbTag, batchTag),
	"RangeKeySet":               makeMethod(rangeKeySetOp{}, dbTag, batchTag),
	"RangeKeyUnset":             makeMethod(rangeKeyUnsetOp{}, dbTag, batchTag),
	"RatchetFormatMajorVersion": makeMethod(dbRatchetFormatMajorVersionOp{}, dbTag),
	"Restart":                   makeMethod(dbRestartOp{}, dbTag),
	"SeekGE":                    makeMethod(iterSeekGEOp{}, iterTag),
	"SeekLT":                    makeMethod(iterSeekLTOp{}, iterTag),
	"SeekPrefixGE":              makeMethod(iterSeekPrefixGEOp{}, iterTag),
	"Set":                       makeMethod(setOp{}, dbTag, batchTag),
	"SetBounds":                 makeMethod(iterSetBoundsOp{}, iterTag),
	"SetOptions":                makeMethod(iterSetOptionsOp{}, iterTag),
	"SingleDelete":              makeMethod(singleDeleteOp{}, dbTag, batchTag),
}

type parser struct {
	fset *token.FileSet
	s    scanner.Scanner
	objs map[objID]bool
}

func parse(src []byte) (_ []op, err error) {
	// Various bits of magic incantation to set up a scanner for Go compatible
	// syntax. We arranged for the textual format of ops (e.g. op.String()) to
	// look like Go which allows us to use the Go scanner for parsing.
	p := &parser{
		fset: token.NewFileSet(),
		objs: map[objID]bool{makeObjID(dbTag, 0): true},
	}
	file := p.fset.AddFile("", -1, len(src))
	p.s.Init(file, src, nil /* no error handler */, 0)
	return p.parse()
}

func (p *parser) parse() (_ []op, err error) {
	defer func() {
		if r := recover(); r != nil {
			var ok bool
			if err, ok = r.(error); ok {
				return
			}
			err = errors.Errorf("%v", r)
		}
	}()

	var ops []op
	for {
		op := p.parseOp()
		if op == nil {
			computeDerivedFields(ops)
			return ops, nil
		}
		ops = append(ops, op)
	}
}

func (p *parser) parseOp() op {
	destPos, destTok, destLit := p.s.Scan()
	if destTok == token.EOF {
		return nil
	}
	if destTok != token.IDENT {
		panic(p.errorf(destPos, "unexpected token: %s %q", destTok, destLit))
	}
	if destLit == "Init" {
		// <op>(<args>)
		return p.makeOp(destLit, makeObjID(dbTag, 0), 0, destPos)
	}

	destID := p.parseObjID(destPos, destLit)

	pos, tok, lit := p.s.Scan()
	switch tok {
	case token.PERIOD:
		// <obj>.<op>(<args>)
		if !p.objs[destID] {
			panic(p.errorf(destPos, "unknown object: %s", destID))
		}
		_, methodLit := p.scanToken(token.IDENT)
		return p.makeOp(methodLit, destID, 0, destPos)

	case token.ASSIGN:
		// <obj> = <obj>.<op>(<args>)
		srcPos, srcLit := p.scanToken(token.IDENT)
		srcID := p.parseObjID(srcPos, srcLit)
		if !p.objs[srcID] {
			panic(p.errorf(srcPos, "unknown object %q", srcLit))
		}
		p.scanToken(token.PERIOD)
		_, methodLit := p.scanToken(token.IDENT)
		p.objs[destID] = true
		return p.makeOp(methodLit, srcID, destID, srcPos)
	}
	panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
}

func (p *parser) parseObjID(pos token.Pos, str string) objID {
	var tag objTag
	switch {
	case str == "db":
		return makeObjID(dbTag, 0)
	case strings.HasPrefix(str, "batch"):
		tag, str = batchTag, str[5:]
	case strings.HasPrefix(str, "iter"):
		tag, str = iterTag, str[4:]
	case strings.HasPrefix(str, "snap"):
		tag, str = snapTag, str[4:]
	default:
		panic(p.errorf(pos, "unable to parse objectID: %q", str))
	}
	id, err := strconv.ParseInt(str, 10, 32)
	if err != nil {
		panic(p.errorf(pos, "%s", err))
	}
	return makeObjID(tag, uint32(id))
}

func unquoteBytes(lit string) []byte {
	s, err := strconv.Unquote(lit)
	if err != nil {
		panic(err)
	}
	if len(s) == 0 {
		return nil
	}
	return []byte(s)
}

func (p *parser) parseArgs(op op, methodName string, args []interface{}) {
	pos, _ := p.scanToken(token.LPAREN)
	for i := range args {
		if i > 0 {
			pos, _ = p.scanToken(token.COMMA)
		}

		switch t := args[i].(type) {
		case *uint32:
			_, lit := p.scanToken(token.INT)
			val, err := strconv.ParseUint(lit, 10, 32)
			if err != nil {
				panic(err)
			}
			*t = uint32(val)

		case *uint64:
			_, lit := p.scanToken(token.INT)
			val, err := strconv.ParseUint(lit, 10, 64)
			if err != nil {
				panic(err)
			}
			*t = uint64(val)

		case *[]byte:
			_, lit := p.scanToken(token.STRING)
			*t = unquoteBytes(lit)

		case *bool:
			_, lit := p.scanToken(token.IDENT)
			b, err := strconv.ParseBool(lit)
			if err != nil {
				panic(err)
			}
			*t = b

		case *objID:
			pos, lit := p.scanToken(token.IDENT)
			*t = p.parseObjID(pos, lit)

		case *[]pebble.KeyRange:
			var pending pebble.KeyRange
			for {
				pos, tok, lit := p.s.Scan()
				switch tok {
				case token.STRING:
					x := unquoteBytes(lit)
					if pending.Start == nil {
						pending.Start = x
					} else {
						pending.End = x
						*t = append(*t, pending)
						pending = pebble.KeyRange{}
					}
					pos, tok, lit := p.s.Scan()
					switch tok {
					case token.COMMA:
						continue
					case token.RPAREN:
						p.scanToken(token.SEMICOLON)
						return
					default:
						panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
					}
				case token.RPAREN:
					p.scanToken(token.SEMICOLON)
					return
				default:
					panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
				}
			}

		case *[]objID:
			for {
				pos, tok, lit := p.s.Scan()
				switch tok {
				case token.IDENT:
					*t = append(*t, p.parseObjID(pos, lit))
					pos, tok, lit := p.s.Scan()
					switch tok {
					case token.COMMA:
						continue
					case token.RPAREN:
						p.scanToken(token.SEMICOLON)
						return
					default:
						panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
					}
				case token.RPAREN:
					p.scanToken(token.SEMICOLON)
					return
				default:
					panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
				}
			}

		case *[]pebble.CheckpointSpan:
			pos, tok, lit := p.s.Scan()
			switch tok {
			case token.RPAREN:
				// No spans.
				*t = nil
				p.scanToken(token.SEMICOLON)
				return

			case token.STRING:
				var keys [][]byte
				for {
					s, err := strconv.Unquote(lit)
					if err != nil {
						panic(p.errorf(pos, "unquoting %q: %v", lit, err))
					}
					keys = append(keys, []byte(s))

					pos, tok, lit = p.s.Scan()
					switch tok {
					case token.COMMA:
						pos, tok, lit = p.s.Scan()
						if tok != token.STRING {
							panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
						}
						continue

					case token.RPAREN:
						p.scanToken(token.SEMICOLON)
						if len(keys)%2 == 1 {
							panic(p.errorf(pos, "expected even number of keys"))
						}
						*t = make([]pebble.CheckpointSpan, len(keys)/2)
						for i := range *t {
							(*t)[i] = pebble.CheckpointSpan{
								Start: keys[i*2],
								End:   keys[i*2+1],
							}
						}
						return

					default:
						panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
					}
				}

			default:
				panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
			}

		case *pebble.FormatMajorVersion:
			_, lit := p.scanToken(token.INT)
			val, err := strconv.ParseUint(lit, 10, 64)
			if err != nil {
				panic(err)
			}
			*t = pebble.FormatMajorVersion(val)

		default:
			panic(p.errorf(pos, "%s: unsupported arg[%d] type: %T", methodName, i, args[i]))
		}
	}
	p.scanToken(token.RPAREN)
	p.scanToken(token.SEMICOLON)
}

func (p *parser) scanToken(expected token.Token) (pos token.Pos, lit string) {
	pos, tok, lit := p.s.Scan()
	if tok != expected {
		panic(p.errorf(pos, "unexpected token: %q", p.tokenf(tok, lit)))
	}
	return pos, lit
}

func (p *parser) makeOp(methodName string, receiverID, targetID objID, pos token.Pos) op {
	info := methods[methodName]
	if info == nil {
		panic(p.errorf(pos, "unknown op %s.%s", receiverID, methodName))
	}
	if info.validTags&(1<<receiverID.tag()) == 0 {
		panic(p.errorf(pos, "%s.%s: %s is not a method on %s",
			receiverID, methodName, methodName, receiverID))
	}

	op := info.constructor()
	receiver, target, args := opArgs(op)

	// The form of an operation is:
	//   [target =] receiver.method(args)
	//
	// The receiver is the object the operation will be called on, which can be
	// any valid ID. Certain operations such as Ingest are only valid on the DB
	// object. That is indicated by opArgs returning a nil receiver.
	if receiver != nil {
		*receiver = receiverID
	} else if receiverID.tag() != dbTag {
		panic(p.errorf(pos, "unknown op %s.%s", receiverID, methodName))
	}

	// The target is the object that will be assigned the result of an object
	// creation operation such as newBatchOp or newIterOp.
	if target != nil {
		// It is invalid to not have a targetID for a method which generates a new
		// object.
		if targetID == 0 {
			panic(p.errorf(pos, "assignment expected for %s.%s", receiverID, methodName))
		}
		// It is invalid to try to assign to the DB object.
		if targetID.tag() == dbTag {
			panic(p.errorf(pos, "cannot use %s as target of assignment", targetID))
		}
		*target = targetID
	} else if targetID != 0 {
		panic(p.errorf(pos, "cannot use %s.%s in assignment", receiverID, methodName))
	}

	p.parseArgs(op, methodName, args)
	return op
}

func (p *parser) tokenf(tok token.Token, lit string) string {
	if tok.IsLiteral() {
		return lit
	}
	return tok.String()
}

func (p *parser) errorf(pos token.Pos, format string, args ...interface{}) error {
	return errors.New(p.fset.Position(pos).String() + ": " + fmt.Sprintf(format, args...))
}

// computeDerivedFields makes one pass through the provided operations, filling
// any derived fields. This pass must happen before execution because concurrent
// execution depends on these fields.
func computeDerivedFields(ops []op) {
	iterToReader := make(map[objID]objID)
	for i := range ops {
		switch v := ops[i].(type) {
		case *newIterOp:
			iterToReader[v.iterID] = v.readerID
		case *newIterUsingCloneOp:
			v.derivedReaderID = iterToReader[v.existingIterID]
			iterToReader[v.iterID] = v.derivedReaderID
		case *iterSetOptionsOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterFirstOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterLastOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterSeekGEOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterSeekPrefixGEOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterSeekLTOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterNextOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterNextPrefixOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterCanSingleDelOp:
			v.derivedReaderID = iterToReader[v.iterID]
		case *iterPrevOp:
			v.derivedReaderID = iterToReader[v.iterID]
		}
	}
}
