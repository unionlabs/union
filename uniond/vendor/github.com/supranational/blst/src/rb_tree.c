/*
 * Copyright Supranational LLC
 * Licensed under the Apache License, Version 2.0, see LICENSE for details.
 * SPDX-License-Identifier: Apache-2.0
 */

#include <stddef.h>

/*
 * Red-black tree tailored for uniqueness test. Amount of messages to be
 * checked is known prior context initialization, implementation is
 * insert-only, failure is returned if message is already in the tree.
 */

struct node {
    struct node *leafs[2];
    const void *data;
    size_t len_n_colour;    /* len<<1 | colour */
};

struct rb_tree {
    struct node *root;
    size_t n_nodes;
    struct node nodes[1];
};

static long bytes_compare(const unsigned char *ptr0, size_t len0,
                          const unsigned char *ptr1, size_t len1)
{
    size_t i, len = len0<len1 ? len0 : len1;
    long a, b;

    for (i=0; i<len; i++) {
        if ((a = ptr0[i]) != (b = ptr1[i]))
            return a - b;
    }

    return (long)len0 - (long)len1;
}

#define PAINT_BLACK(p)  ((p)->len_n_colour &= ~(size_t)1)
#define PAINT_RED(p)    ((p)->len_n_colour |= 1)
#define IS_RED(p)       ((p)->len_n_colour & 1)

static int rb_tree_insert(struct rb_tree *tree, const void *data, size_t len)
{
    struct node *nodes[8*sizeof(void *)];   /* visited nodes    */
    unsigned char dirs[8*sizeof(void *)];   /* taken directions */
    size_t k = 0;                           /* walked distance  */
    struct node *p, *y, *z;

    for (p = tree->root; p != NULL; k++) {
        long cmp = bytes_compare(data, len, p->data, p->len_n_colour>>1);

        if (cmp == 0)
            return 0;   /* already in tree, no insertion */

        /* record the step */
        nodes[k] = p;
        p = p->leafs[(dirs[k] = cmp>0)];
    }

    /* allocate new node */
    z = &tree->nodes[tree->n_nodes++];
    z->leafs[0] = z->leafs[1] = NULL;
    z->data = data;
    z->len_n_colour = len<<1;
    PAINT_RED(z);

    /* graft |z| */
    if (k > 0)
        nodes[k-1]->leafs[dirs[k-1]] = z;
    else
        tree->root = z;

    /* re-balance |tree| */
    while (k >= 2 && IS_RED(y = nodes[k-1])) {
        size_t ydir = dirs[k-2];
        struct node *x = nodes[k-2],        /* |z|'s grandparent    */
                    *s = x->leafs[ydir^1];  /* |z|'s uncle          */

        if (s != NULL && IS_RED(s)) {
            PAINT_RED(x);
            PAINT_BLACK(y);
            PAINT_BLACK(s);
            k -= 2;
        } else {
            if (dirs[k-1] != ydir) {
                /*    |        |
                 *    x        x
                 *   / \        \
                 *  y   s -> z   s
                 *   \      /
                 *    z    y
                 *   /      \
                 *  ?        ?
                 */
                struct node *t = y;
                y = y->leafs[ydir^1];
                t->leafs[ydir^1] = y->leafs[ydir];
                y->leafs[ydir] = t;
            }

            /*      |        |
             *      x        y
             *       \      / \
             *    y   s -> z   x
             *   / \          / \
             *  z   ?        ?   s
             */
            x->leafs[ydir] = y->leafs[ydir^1];
            y->leafs[ydir^1] = x;

            PAINT_RED(x);
            PAINT_BLACK(y);

            if (k > 2)
                nodes[k-3]->leafs[dirs[k-3]] = y;
            else
                tree->root = y;

            break;
        }
    }

    PAINT_BLACK(tree->root);

    return 1;
}

#undef IS_RED
#undef PAINT_RED
#undef PAINT_BLACK

size_t blst_uniq_sizeof(size_t n_nodes)
{   return sizeof(struct rb_tree) + sizeof(struct node)*(n_nodes-1);   }

void blst_uniq_init(struct rb_tree *tree)
{
    tree->root = NULL;
    tree->n_nodes = 0;
}

int blst_uniq_test(struct rb_tree *tree, const void *data, size_t len)
{   return (int)rb_tree_insert(tree, data, len);   }
