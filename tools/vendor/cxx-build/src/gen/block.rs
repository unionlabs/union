use proc_macro2::Ident;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Block<'a> {
    AnonymousNamespace,
    Namespace(&'static str),
    UserDefinedNamespace(&'a Ident),
    InlineNamespace(&'static str),
    ExternC,
}

impl<'a> Block<'a> {
    pub fn write_begin(self, out: &mut String) {
        if let Block::InlineNamespace(_) = self {
            out.push_str("inline ");
        }
        self.write_common(out);
        out.push_str(" {\n");
    }

    pub fn write_end(self, out: &mut String) {
        out.push_str("} // ");
        self.write_common(out);
        out.push('\n');
    }

    fn write_common(self, out: &mut String) {
        match self {
            Block::AnonymousNamespace => out.push_str("namespace"),
            Block::Namespace(name) => {
                out.push_str("namespace ");
                out.push_str(name);
            }
            Block::UserDefinedNamespace(name) => {
                out.push_str("namespace ");
                out.push_str(&name.to_string());
            }
            Block::InlineNamespace(name) => {
                out.push_str("namespace ");
                out.push_str(name);
            }
            Block::ExternC => out.push_str("extern \"C\""),
        }
    }
}
