//! `refill` is a small template library based on [`facet`](https://github.com/facet-rs/facet).
//!
//! ## Example
//!
//! ```rust
//! use facet::Facet;
//! use refill::Template;
//!
//! #[derive(Facet)]
//! struct Custom {
//!     special_field: f64,
//!     special_name: String,
//! }
//!
//! #[derive(Facet)]
//! struct Ctx<'a> {
//!     name: String,
//!     custom: &'a Custom,
//!     multi: Vec<usize>,
//! }
//!
//! let c = Custom {
//!     special_field: 42.1,
//!     special_name: String::from("Also Seth"),
//! };
//! let ctx = Ctx {
//!     name: String::from("Seth"),
//!     custom: &c,
//!     multi: vec![0, 1, 2, 3],
//! };
//!
//! let found = Template::new("Hello, my name is {{custom.special_name}}.").fill(&ctx).unwrap();
//! let expected = "Hello, my name is Also Seth.";
//! assert_eq!(found, expected);
//! ```
//!
//! ## Key Features
//!
//! - Supports nested struct field paths.
//! - Formats based on the `Display` impl for the type in question.
//! - Fields on the passed-in `ctx` type are used as the values in the templates.
pub mod error;

use crate::error::{TemplateError, TemplateErrorKind};
use facet::{Facet, Peek, PointerType, Type};

/// The parts of the template.
///
/// Effectively tokens.
#[derive(Facet)]
#[repr(u8)]
enum Chunk<'a> {
    Text(&'a str),
    Expr(&'a str),
}

/// A template that can be filled in, given a context struct.
#[derive(Facet)]
pub struct Template<'a> {
    /// The raw template string
    raw: &'a str,
    /// The parsed template string broken up into tokens
    chunks: Vec<Chunk<'a>>,
}

impl<'a> Template<'a> {
    /// Create a new template.
    pub fn new(tmpl: &'a str) -> Self {
        Self {
            raw: tmpl,
            chunks: parse_tmpl(tmpl),
        }
    }

    /// Fill in a template using the fields on the `ctx` struct.
    pub fn fill<'mem, 'facet, Ctx: Facet<'facet> + ?Sized>(
        &self,
        ctx: &'mem Ctx,
    ) -> Result<String, TemplateError> {
        fill_tmpl(&self.chunks, ctx)
    }
}

/// Parse the template into [`Chunk`]s.
fn parse_tmpl(tmpl: &str) -> Vec<Chunk<'_>> {
    let mut chunks = vec![];

    let mut i = 0;
    for (chunk_start, _) in tmpl.match_indices("{{") {
        if chunk_start.saturating_sub(i) > 0 {
            chunks.push(Chunk::Text(&tmpl[i..chunk_start]))
        }
        if let Some(end) = tmpl[chunk_start..].find("}}") {
            chunks.push(Chunk::Expr(&tmpl[chunk_start + 2..chunk_start + end]));
            i = chunk_start + end + 2;
        }
    }
    if i < tmpl.len() {
        chunks.push(Chunk::Text(&tmpl[i..]));
    }

    // for (idx, c) in tmpl.char_indices() {
    //     if c == '{' {
    //         in_expr = true;
    //         chunks.push(Chunk::Text(&tmpl[start..idx]));
    //         start = idx + 1;
    //     } else if in_expr && c == '}' {
    //         in_expr = false;
    //         chunks.push(Chunk::Expr(&tmpl[start..idx]));
    //         start = idx + 1;
    //     }
    // }
    // if start < tmpl.len() {
    //     // Can only be text since an expr would have hit the closing bracket.
    //     chunks.push(Chunk::Text(&tmpl[start..]));
    // }
    chunks
}

/// Fill in the template, following paths.
fn fill_tmpl<'facet, 'mem, Ctx: Facet<'facet> + ?Sized>(
    chunks: &[Chunk<'_>],
    ctx: &'mem Ctx,
) -> Result<String, TemplateError> {
    let mut ret = String::new();

    for chunk in chunks {
        match chunk {
            Chunk::Text(text) => ret.push_str(text),
            Chunk::Expr(ident) => {
                let mut value = deref_refs(Peek::new(ctx));

                for part in ident.split('.') {
                    value = deref_refs(
                        value
                            .into_struct()
                            .map_err(|e| TemplateError {
                                kind: TemplateErrorKind::InvalidContextError { reflect_error: e },
                            })?
                            .field_by_name(part)
                            .map_err(|_e| TemplateError {
                                kind: TemplateErrorKind::MissingFieldError {
                                    expected: (*ident).to_owned(),
                                },
                            })?,
                    );
                }
                ret.push_str(&value.to_string());
            }
        }
    }
    Ok(ret)
}

/// Step past refs for when a field or input struct is a `&` or `&&&&&` etc.
fn deref_refs<'mem, 'facet>(mut peek: Peek<'mem, 'facet>) -> Peek<'mem, 'facet> {
    loop {
        if !matches!(peek.shape().ty, Type::Pointer(PointerType::Reference(_))) {
            return peek;
        }

        let Ok(ptr) = peek.into_pointer() else {
            return peek;
        };
        let Some(inner) = ptr.borrow_inner() else {
            return peek;
        };
        peek = inner;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rediff::assert_same;

    #[test]
    fn simple_parse_impl() {
        let template = "Hello {{name}}!";
        let chunks = parse_tmpl(template);
        let expected = vec![Chunk::Text("Hello "), Chunk::Expr("name"), Chunk::Text("!")];
        assert_same!(expected, chunks);
    }

    #[test]
    fn simple_fill_tmpl() {
        #[derive(Facet)]
        struct Ctx {
            name: String,
        }

        let ctx = Ctx {
            name: "Vader".to_string(),
        };

        let template = "Hello {{name}}!";
        let chunks = parse_tmpl(template);
        // Note: the && is on purpose to check we handle ref-following correctly
        let ret = fill_tmpl(&chunks, &&ctx).unwrap();
        assert_eq!(ret, "Hello Vader!");
    }
}
