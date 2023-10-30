#![allow(clippy::branches_sharing_code)]

#[cfg(test)]
mod tests;

use std::collections::VecDeque;
use std::fmt::{self, Display};
use std::rc::Rc;

/// a simple recursive type which is able to render its
/// components in a tree-like format
#[derive(Debug, Clone)]
pub struct Tree<D: Display> {
    pub root: D,
    pub leaves: Vec<Tree<D>>,
    multiline: bool,
    glyphs: GlyphPalette,
}

impl<D: Display> Tree<D> {
    pub fn new(root: D) -> Self {
        Tree {
            root,
            leaves: Vec::new(),
            multiline: false,
            glyphs: GlyphPalette::new(),
        }
    }

    pub fn with_leaves(mut self, leaves: impl IntoIterator<Item = impl Into<Tree<D>>>) -> Self {
        self.leaves = leaves.into_iter().map(Into::into).collect();
        self
    }

    /// Ensure all lines for `root` are indented
    pub fn with_multiline(mut self, yes: bool) -> Self {
        self.multiline = yes;
        self
    }

    /// Customize the rendering of this node
    pub fn with_glyphs(mut self, glyphs: GlyphPalette) -> Self {
        self.glyphs = glyphs;
        self
    }
}

impl<D: Display> Tree<D> {
    /// Ensure all lines for `root` are indented
    pub fn set_multiline(&mut self, yes: bool) -> &mut Self {
        self.multiline = yes;
        self
    }

    /// Customize the rendering of this node
    pub fn set_glyphs(&mut self, glyphs: GlyphPalette) -> &mut Self {
        self.glyphs = glyphs;
        self
    }
}

impl<D: Display> Tree<D> {
    pub fn push(&mut self, leaf: impl Into<Tree<D>>) -> &mut Self {
        self.leaves.push(leaf.into());
        self
    }
}

impl<D: Display> From<D> for Tree<D> {
    fn from(inner: D) -> Self {
        Self::new(inner)
    }
}

impl<D: Display> Extend<D> for Tree<D> {
    fn extend<T: IntoIterator<Item = D>>(&mut self, iter: T) {
        self.leaves.extend(iter.into_iter().map(Into::into))
    }
}

impl<D: Display> Extend<Tree<D>> for Tree<D> {
    fn extend<T: IntoIterator<Item = Tree<D>>>(&mut self, iter: T) {
        self.leaves.extend(iter)
    }
}

impl<D: Display> Display for Tree<D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.root.fmt(f)?; // Pass along `f.alternate()`
        writeln!(f)?;
        let mut queue = DisplauQueue::new();
        let no_space = Rc::new(Vec::new());
        enqueue_leaves(&mut queue, self, no_space);
        while let Some((last, leaf, spaces)) = queue.pop_front() {
            let mut prefix = (
                if last {
                    leaf.glyphs.last_item
                } else {
                    leaf.glyphs.middle_item
                },
                leaf.glyphs.item_indent,
            );

            if leaf.multiline {
                let rest_prefix = (
                    if last {
                        leaf.glyphs.last_skip
                    } else {
                        leaf.glyphs.middle_skip
                    },
                    leaf.glyphs.skip_indent,
                );
                debug_assert_eq!(prefix.0.chars().count(), rest_prefix.0.chars().count());
                debug_assert_eq!(prefix.1.chars().count(), rest_prefix.1.chars().count());

                let root = if f.alternate() {
                    format!("{:#}", leaf.root)
                } else {
                    format!("{:}", leaf.root)
                };
                for line in root.lines() {
                    // print single line
                    for s in spaces.as_slice() {
                        if *s {
                            self.glyphs.last_skip.fmt(f)?;
                            self.glyphs.skip_indent.fmt(f)?;
                        } else {
                            self.glyphs.middle_skip.fmt(f)?;
                            self.glyphs.skip_indent.fmt(f)?;
                        }
                    }
                    prefix.0.fmt(f)?;
                    prefix.1.fmt(f)?;
                    line.fmt(f)?;
                    writeln!(f)?;
                    prefix = rest_prefix;
                }
            } else {
                // print single line
                for s in spaces.as_slice() {
                    if *s {
                        self.glyphs.last_skip.fmt(f)?;
                        self.glyphs.skip_indent.fmt(f)?;
                    } else {
                        self.glyphs.middle_skip.fmt(f)?;
                        self.glyphs.skip_indent.fmt(f)?;
                    }
                }
                prefix.0.fmt(f)?;
                prefix.1.fmt(f)?;
                leaf.root.fmt(f)?; // Pass along `f.alternate()`
                writeln!(f)?;
            }

            // recurse
            if !leaf.leaves.is_empty() {
                let s: &Vec<bool> = &spaces;
                let mut child_spaces = s.clone();
                child_spaces.push(last);
                let child_spaces = Rc::new(child_spaces);
                enqueue_leaves(&mut queue, leaf, child_spaces);
            }
        }
        Ok(())
    }
}

type DisplauQueue<'t, D> = VecDeque<(bool, &'t Tree<D>, Rc<Vec<bool>>)>;

fn enqueue_leaves<'t, D: Display>(
    queue: &mut DisplauQueue<'t, D>,
    parent: &'t Tree<D>,
    spaces: Rc<Vec<bool>>,
) {
    for (i, leaf) in parent.leaves.iter().rev().enumerate() {
        let last = i == 0;
        queue.push_front((last, leaf, spaces.clone()));
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct GlyphPalette {
    pub middle_item: &'static str,
    pub last_item: &'static str,
    pub item_indent: &'static str,

    pub middle_skip: &'static str,
    pub last_skip: &'static str,
    pub skip_indent: &'static str,
}

impl GlyphPalette {
    pub const fn new() -> Self {
        Self {
            middle_item: "├",
            last_item: "└",
            item_indent: "── ",

            middle_skip: "│",
            last_skip: " ",
            skip_indent: "   ",
        }
    }
}

impl Default for GlyphPalette {
    fn default() -> Self {
        Self::new()
    }
}
