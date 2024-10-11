/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */
/*
 * Reimplement rb_tree.c, because C.call overhead is too high in
 * comparison to tree insertion subroutine.
 */

package blst

import "bytes"

/*
 * Red-black tree tailored for uniqueness test. Amount of messages to be
 * checked is known prior context initialization, implementation is
 * insert-only, failure is returned if message is already in the tree.
 */

const red, black bool = true, false

type node struct {
    leafs  [2]*node
    data   *[]byte
    colour bool
}

type rbTree struct {
    root   *node
    nnodes uint
    nodes  []node
}

func (tree *rbTree) insert(data *[]byte) bool {
    var nodes [64]*node     /* visited nodes    */
    var dirs  [64]byte      /* taken directions */
    var k uint              /* walked distance  */

    for p := tree.root; p != nil; k++ {
        cmp := bytes.Compare(*data, *p.data)

        if cmp == 0 {
            return false    /* already in tree, no insertion */
        }

        /* record the step */
        nodes[k] = p
        if cmp > 0 {
            dirs[k] = 1
        } else {
            dirs[k] = 0
        }
        p = p.leafs[dirs[k]]
    }

    /* allocate new node */
    z := &tree.nodes[tree.nnodes]; tree.nnodes++
    z.data = data
    z.colour = red

    /* graft |z| */
    if k > 0 {
        nodes[k-1].leafs[dirs[k-1]] = z
    } else {
        tree.root = z
    }

    /* re-balance |tree| */
    for k >= 2 /* && IS_RED(y = nodes[k-1]) */ {
        y := nodes[k-1]
        if y.colour == black  { //nolint:gosimple
            break
        }

        ydir := dirs[k-2]
        x := nodes[k-2]         /* |z|'s grandparent    */
        s := x.leafs[ydir^1]    /* |z|'s uncle          */

        if s != nil && s.colour == red { //nolint:gosimple,revive
            x.colour = red
            y.colour = black
            s.colour = black
            k -= 2
        } else {
            if dirs[k-1] != ydir {
                /*    |        |
                 *    x        x
                 *   / \        \
                 *  y   s -> z   s
                 *   \      /
                 *    z    y
                 *   /      \
                 *  ?        ?
                 */
                t := y
                y = y.leafs[ydir^1]
                t.leafs[ydir^1] = y.leafs[ydir]
                y.leafs[ydir] = t
            }

            /*      |        |
             *      x        y
             *       \      / \
             *    y   s -> z   x
             *   / \          / \
             *  z   ?        ?   s
             */
            x.leafs[ydir] = y.leafs[ydir^1]
            y.leafs[ydir^1] = x

            x.colour = red
            y.colour = black

            if k > 2 {
                nodes[k-3].leafs[dirs[k-3]] = y
            } else {
                tree.root = y
            }

            break
        }
    }

    tree.root.colour = black

    return true
}

func Uniq(msgs []Message) bool {
    n := len(msgs)

    if n == 1 {
        return true
    } else if n == 2 {
        return !bytes.Equal(msgs[0], msgs[1])
    }

    var tree rbTree
    tree.nodes = make([]node, n)

    for i := 0; i < n; i++ {
        if !tree.insert(&msgs[i]) {
            return false
        }
    }

    return true
}
