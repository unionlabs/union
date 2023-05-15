/*
Package merkle computes a deterministic minimal height Merkle tree hash.
If the number of items is not a power of two, some leaves
will be at different levels. Tries to keep both sides of
the tree the same size, but the left may be one greater.

Use this for short deterministic trees, such as the validator list.
For larger datasets, use IAVLTree.

Be aware that the current implementation by itself does not prevent
second pre-image attacks. Hence, use this library with caution.
Otherwise you might run into similar issues as, e.g., in early Bitcoin:
https://bitcointalk.org/?topic=102395

	              *
	             / \
	           /     \
	         /         \
	       /             \
	      *               *
	     / \             / \
	    /   \           /   \
	   /     \         /     \
	  *       *       *       h6
	 / \     / \     / \
	h0  h1  h2  h3  h4  h5

TODO(ismail): add 2nd pre-image protection or clarify further on how we use this and why this secure.
*/
package merkle
