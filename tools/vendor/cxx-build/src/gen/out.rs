use crate::gen::block::Block;
use crate::gen::builtin::Builtins;
use crate::gen::include::Includes;
use crate::gen::Opt;
use crate::syntax::namespace::Namespace;
use crate::syntax::Types;
use std::cell::RefCell;
use std::fmt::{self, Arguments, Write};

pub(crate) struct OutFile<'a> {
    pub header: bool,
    pub opt: &'a Opt,
    pub types: &'a Types<'a>,
    pub include: Includes<'a>,
    pub builtin: Builtins<'a>,
    content: RefCell<Content<'a>>,
}

#[derive(Default)]
pub struct Content<'a> {
    bytes: String,
    namespace: &'a Namespace,
    blocks: Vec<BlockBoundary<'a>>,
    section_pending: bool,
    blocks_pending: usize,
}

#[derive(Copy, Clone, PartialEq, Debug)]
enum BlockBoundary<'a> {
    Begin(Block<'a>),
    End(Block<'a>),
}

impl<'a> OutFile<'a> {
    pub fn new(header: bool, opt: &'a Opt, types: &'a Types) -> Self {
        OutFile {
            header,
            opt,
            types,
            include: Includes::new(),
            builtin: Builtins::new(),
            content: RefCell::new(Content::new()),
        }
    }

    // Write a blank line if the preceding section had any contents.
    pub fn next_section(&mut self) {
        self.content.get_mut().next_section();
    }

    pub fn begin_block(&mut self, block: Block<'a>) {
        self.content.get_mut().begin_block(block);
    }

    pub fn end_block(&mut self, block: Block<'a>) {
        self.content.get_mut().end_block(block);
    }

    pub fn set_namespace(&mut self, namespace: &'a Namespace) {
        self.content.get_mut().set_namespace(namespace);
    }

    pub fn write_fmt(&self, args: Arguments) {
        let content = &mut *self.content.borrow_mut();
        Write::write_fmt(content, args).unwrap();
    }

    pub fn content(&mut self) -> Vec<u8> {
        self.flush();
        let include = &self.include.content.bytes;
        let builtin = &self.builtin.content.bytes;
        let content = &self.content.get_mut().bytes;
        let len = include.len() + builtin.len() + content.len() + 2;
        let mut out = String::with_capacity(len);
        out.push_str(include);
        if !out.is_empty() && !builtin.is_empty() {
            out.push('\n');
        }
        out.push_str(builtin);
        if !out.is_empty() && !content.is_empty() {
            out.push('\n');
        }
        out.push_str(content);
        if out.is_empty() {
            out.push_str("// empty\n");
        }
        out.into_bytes()
    }

    fn flush(&mut self) {
        self.include.content.flush();
        self.builtin.content.flush();
        self.content.get_mut().flush();
    }
}

impl<'a> Write for Content<'a> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write(s);
        Ok(())
    }
}

impl<'a> PartialEq for Content<'a> {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl<'a> Content<'a> {
    fn new() -> Self {
        Content::default()
    }

    pub fn next_section(&mut self) {
        self.section_pending = true;
    }

    pub fn begin_block(&mut self, block: Block<'a>) {
        self.push_block_boundary(BlockBoundary::Begin(block));
    }

    pub fn end_block(&mut self, block: Block<'a>) {
        self.push_block_boundary(BlockBoundary::End(block));
    }

    pub fn set_namespace(&mut self, namespace: &'a Namespace) {
        for name in self.namespace.iter().rev() {
            self.end_block(Block::UserDefinedNamespace(name));
        }
        for name in namespace {
            self.begin_block(Block::UserDefinedNamespace(name));
        }
        self.namespace = namespace;
    }

    pub fn write_fmt(&mut self, args: Arguments) {
        Write::write_fmt(self, args).unwrap();
    }

    fn write(&mut self, b: &str) {
        if !b.is_empty() {
            if self.blocks_pending > 0 {
                self.flush_blocks();
            }
            if self.section_pending && !self.bytes.is_empty() {
                self.bytes.push('\n');
            }
            self.bytes.push_str(b);
            self.section_pending = false;
            self.blocks_pending = 0;
        }
    }

    fn push_block_boundary(&mut self, boundary: BlockBoundary<'a>) {
        if self.blocks_pending > 0 && boundary == self.blocks.last().unwrap().rev() {
            self.blocks.pop();
            self.blocks_pending -= 1;
        } else {
            self.blocks.push(boundary);
            self.blocks_pending += 1;
        }
    }

    fn flush(&mut self) {
        self.set_namespace(Default::default());
        if self.blocks_pending > 0 {
            self.flush_blocks();
        }
    }

    fn flush_blocks(&mut self) {
        self.section_pending = !self.bytes.is_empty();
        let mut read = self.blocks.len() - self.blocks_pending;
        let mut write = read;

        while read < self.blocks.len() {
            match self.blocks[read] {
                BlockBoundary::Begin(begin_block) => {
                    if self.section_pending {
                        self.bytes.push('\n');
                        self.section_pending = false;
                    }
                    Block::write_begin(begin_block, &mut self.bytes);
                    self.blocks[write] = BlockBoundary::Begin(begin_block);
                    write += 1;
                }
                BlockBoundary::End(end_block) => {
                    write = write.checked_sub(1).unwrap();
                    let begin_block = self.blocks[write];
                    assert_eq!(begin_block, BlockBoundary::Begin(end_block));
                    Block::write_end(end_block, &mut self.bytes);
                    self.section_pending = true;
                }
            }
            read += 1;
        }

        self.blocks.truncate(write);
    }
}

impl<'a> BlockBoundary<'a> {
    fn rev(self) -> BlockBoundary<'a> {
        match self {
            BlockBoundary::Begin(block) => BlockBoundary::End(block),
            BlockBoundary::End(block) => BlockBoundary::Begin(block),
        }
    }
}
