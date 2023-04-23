# B-tree Path Hints

I use a thing I call path hints in my B-tree [C](https://github.com/tidwall/btree.c) and [Go](https://github.com/tidwall/btree) implementations. It's a search optimization.

## The B-tree

A standard [B-tree](https://en.wikipedia.org/wiki/B-tree) is an ordered tree-based data structure that stores its items in nodes. The B-tree has a single root node, which may have children nodes, and those children nodes may also have children nodes. 

<img width="322" alt="image" src="https://user-images.githubusercontent.com/1156077/127664015-14ca38bb-1a3b-4d2f-80ff-27be0bd3d886.png">

Searching for items in a B-tree is fast. [O(log N)](https://en.wikipedia.org/wiki/Big_O_notation) to be exact.
This is because the [binary search algorithm](https://en.wikipedia.org/wiki/Binary_search_algorithm) is used. 

A binary search works by first comparing the item at the middle-most index of the root node with the target item. 
If the middle item is greater than the target item, then it divides the node in two and does the binary search on the left part of the node. If the middle is less, it searches the right part. And so on. If the target item is found, then the search stop. If the item is not found, then the search is passed to the child node at the appropriate index. This traversal terminates when item is found or there are no more child nodes.

<img width="600" alt="image" src="https://user-images.githubusercontent.com/1156077/127664822-6ab4f8f6-8ab5-477e-8e17-f52346f02819.png">

## The Path

Each index is a component of the path to the item (or where the item should be stored, if it does not exist in the tree).

Take the first example image. The item 9 is at path “1/0”. The item 16 is at path “1”. The item 21 is at path “2/1”. The item 5 is at path “0/2”.

## The Path Hint

A Path Hint is a predefined path that is provided to B-tree operations. It’s just a hint that says, “Hey B-tree, instead of starting your binary search with the middle index, start with what I provide you. My path may be wrong, and if so please provide me with the correct path so I get it right the next time.”

I’ve found using path hints can lead to a little performance increase of 150% - 300%. This is because in real-world cases the items that I’m working with are usually nearby each other in the tree.

Take for example inserting a group of timeseries points. They may often be received as chucks of near-contiguous items.  
Or, I'm sequentially inserting an ordered group of rows somewhere in the middle of a table.  
Or, I have a Redis-style key/value store, where the keys look have the common structure “user:98512:name”, “user:98512:email”, and I want to update a bunch of values for specified user.  
Using a path hint may help to avoid the unnecessary binary searching in each of these examples.

While I may see a 3x boost in when the path hint is right on, I'll only see around 5% decrease when the path hint is totally wrong.

## Using a Path Hint

All of the functions that take in a path hint argument mutate the path hint argument.

For single-threaded programs, it’s possible to use one shared path hint per B-tree for the life of the program.  
For multi-threaded programs, I find it best to use one path hint per B-tree , per thread.  
For server-client programs, one path hint per B-tree, per client should suffice.  

