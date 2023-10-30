use proc_macro2::TokenStream;

#[derive(Debug, Clone)]
pub struct SchemaMetadata<'a> {
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub deprecated: bool,
    pub read_only: bool,
    pub write_only: bool,
    pub examples: &'a [syn::Path],
    pub default: Option<TokenStream>,
}

impl<'a> SchemaMetadata<'a> {
    pub fn apply_to_schema(&self, schema_expr: &mut TokenStream) {
        let setters = self.make_setters();
        if !setters.is_empty() {
            *schema_expr = quote! {
                schemars::_private::apply_metadata(#schema_expr, schemars::schema::Metadata {
                    #(#setters)*
                    ..Default::default()
                })
            }
        }
    }

    fn make_setters(&self) -> Vec<TokenStream> {
        let mut setters = Vec::<TokenStream>::new();

        if let Some(title) = &self.title {
            setters.push(quote! {
                title: Some(#title.to_owned()),
            });
        }
        if let Some(description) = &self.description {
            setters.push(quote! {
                description: Some(#description.to_owned()),
            });
        }

        if self.deprecated {
            setters.push(quote! {
                deprecated: true,
            });
        }

        if self.read_only {
            setters.push(quote! {
                read_only: true,
            });
        }
        if self.write_only {
            setters.push(quote! {
                write_only: true,
            });
        }

        if !self.examples.is_empty() {
            let examples = self.examples.iter().map(|eg| {
                quote! {
                    schemars::_serde_json::value::to_value(#eg())
                }
            });
            setters.push(quote! {
                examples: vec![#(#examples),*].into_iter().flatten().collect(),
            });
        }

        if let Some(default) = &self.default {
            setters.push(quote! {
                default: #default.and_then(|d| schemars::_schemars_maybe_to_value!(d)),
            });
        }

        setters
    }
}
