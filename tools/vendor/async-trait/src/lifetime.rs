use proc_macro2::{Span, TokenStream};
use std::mem;
use syn::visit_mut::{self, VisitMut};
use syn::{
    parse_quote_spanned, token, Expr, GenericArgument, Lifetime, Receiver, ReturnType, Type,
    TypeBareFn, TypeImplTrait, TypeParen, TypePtr, TypeReference,
};

pub struct CollectLifetimes {
    pub elided: Vec<Lifetime>,
    pub explicit: Vec<Lifetime>,
    pub name: &'static str,
    pub default_span: Span,
}

impl CollectLifetimes {
    pub fn new(name: &'static str, default_span: Span) -> Self {
        CollectLifetimes {
            elided: Vec::new(),
            explicit: Vec::new(),
            name,
            default_span,
        }
    }

    fn visit_opt_lifetime(&mut self, lifetime: &mut Option<Lifetime>) {
        match lifetime {
            None => *lifetime = Some(self.next_lifetime(None)),
            Some(lifetime) => self.visit_lifetime(lifetime),
        }
    }

    fn visit_lifetime(&mut self, lifetime: &mut Lifetime) {
        if lifetime.ident == "_" {
            *lifetime = self.next_lifetime(lifetime.span());
        } else {
            self.explicit.push(lifetime.clone());
        }
    }

    fn next_lifetime<S: Into<Option<Span>>>(&mut self, span: S) -> Lifetime {
        let name = format!("{}{}", self.name, self.elided.len());
        let span = span.into().unwrap_or(self.default_span);
        let life = Lifetime::new(&name, span);
        self.elided.push(life.clone());
        life
    }
}

impl VisitMut for CollectLifetimes {
    fn visit_receiver_mut(&mut self, arg: &mut Receiver) {
        if let Some((_, lifetime)) = &mut arg.reference {
            self.visit_opt_lifetime(lifetime);
        }
    }

    fn visit_type_reference_mut(&mut self, ty: &mut TypeReference) {
        self.visit_opt_lifetime(&mut ty.lifetime);
        visit_mut::visit_type_reference_mut(self, ty);
    }

    fn visit_generic_argument_mut(&mut self, gen: &mut GenericArgument) {
        if let GenericArgument::Lifetime(lifetime) = gen {
            self.visit_lifetime(lifetime);
        }
        visit_mut::visit_generic_argument_mut(self, gen);
    }
}

pub struct AddLifetimeToImplTrait;

impl VisitMut for AddLifetimeToImplTrait {
    fn visit_type_impl_trait_mut(&mut self, ty: &mut TypeImplTrait) {
        let span = ty.impl_token.span;
        let lifetime = parse_quote_spanned!(span=> 'async_trait);
        ty.bounds.insert(0, lifetime);
        if let Some(punct) = ty.bounds.pairs_mut().next().unwrap().punct_mut() {
            punct.span = span;
        }
        visit_mut::visit_type_impl_trait_mut(self, ty);
    }

    fn visit_type_reference_mut(&mut self, ty: &mut TypeReference) {
        parenthesize_impl_trait(&mut ty.elem, ty.and_token.span);
        visit_mut::visit_type_reference_mut(self, ty);
    }

    fn visit_type_ptr_mut(&mut self, ty: &mut TypePtr) {
        parenthesize_impl_trait(&mut ty.elem, ty.star_token.span);
        visit_mut::visit_type_ptr_mut(self, ty);
    }

    fn visit_type_bare_fn_mut(&mut self, ty: &mut TypeBareFn) {
        if let ReturnType::Type(arrow, return_type) = &mut ty.output {
            parenthesize_impl_trait(return_type, arrow.spans[0]);
        }
        visit_mut::visit_type_bare_fn_mut(self, ty);
    }

    fn visit_expr_mut(&mut self, _e: &mut Expr) {
        // Do not recurse into impl Traits inside of an array length expression.
        //
        //    fn outer(arg: [u8; { fn inner(_: impl Trait) {}; 0 }]);
    }
}

fn parenthesize_impl_trait(elem: &mut Type, paren_span: Span) {
    if let Type::ImplTrait(_) = *elem {
        let placeholder = Type::Verbatim(TokenStream::new());
        *elem = Type::Paren(TypeParen {
            paren_token: token::Paren(paren_span),
            elem: Box::new(mem::replace(elem, placeholder)),
        });
    }
}
