pub mod error;

use crate::error::{TemplateError, TemplateErrorKind};
use facet::{Facet, Peek, PointerType, Type};

#[derive(Facet)]
#[repr(u8)]
enum Chunk<'a> {
    Text(&'a str),
    Expr(&'a str),
    If {
        cond: &'a str,
        body: Vec<Chunk<'a>>,
        ifelse: Option<Vec<IfElse<'a>>>,
        end_else: Option<Vec<Chunk<'a>>>,
    },
}

#[derive(Facet)]
struct IfElse<'a> {
    cond: &'a str,
    body: Vec<Chunk<'a>>,
}

#[derive(Facet)]
pub struct Template<'a> {
    raw: &'a str,
    chunks: Vec<Chunk<'a>>,
}

impl<'a> Template<'a> {
    pub fn new(tmpl: &'a str) -> Self {
        Self {
            raw: tmpl,
            chunks: parse_tmpl(tmpl),
        }
    }

    pub fn fill<'mem, 'facet, Ctx: Facet<'facet> + ?Sized>(
        &self,
        ctx: &'mem Ctx,
    ) -> Result<String, TemplateError> {
        fill_tmpl(&self.chunks, ctx)
    }
}

fn parse_tmpl(tmpl: &str) -> Vec<Chunk<'_>> {
    let mut chunks = vec![];
    let mut start = 0;
    let mut in_expr = false;
    let mut iter = tmpl.char_indices().peekable();

    while let Some((idx, c)) = iter.next() {}
    for (idx, c) in tmpl.char_indices() {
        if c == '{' {
            in_expr = true;
            chunks.push(Chunk::Text(&tmpl[start..idx]));
            start = idx + 1;
        } else if in_expr && c == '}' {
            in_expr = false;
            chunks.push(Chunk::Expr(&tmpl[start..idx]));
            start = idx + 1;
        }
    }
    if start < tmpl.len() {
        // Can only be text since an expr would have hit the closing bracket.
        chunks.push(Chunk::Text(&tmpl[start..]));
    }
    chunks
}

struct Consumed<'a> {
    chunk: Chunk<'a>,
    amount: usize,
}
impl<'a> Consumed<'a> {
    fn new(chunk: Chunk<'a>, amount: usize) -> Self {
        Self { chunk, amount }
    }
}

fn try_parse_ident(tmpl: &str) -> Option<Consumed<'_>> {
    if !tmpl.starts_with('{') {
        return None;
    }
    let Some(end) = tmpl.find('}') else {
        return None;
    };

    tmpl.get(1..end).map(|s| Consumed::new(Chunk::Expr(s), end))
}

//{#if foo}Foo is True{#endif}
//{#if !foo}Foo is not True{#else}Foo is True{#endif}
//{#if !foo}Foo is notTrue{#elseif bar}Bar is True{#elseif car}Car is True{#endif}
fn try_parse_if(tmpl: &str) -> Option<Consumed<'_>> {
    if !tmpl.starts_with("{# if") {
        return None;
    }
    let Some(if_end) = tmpl.find("}") else {
        return None;
    };

    let Some(cond) = tmpl.get(5..if_end).map(|s| s.trim()) else {
        return None;
    };

    let Some(overall_end) = tmpl.find("{# endif #}") else {
        return None;
    };

    // Check for elifs
    for (i, _) in tmpl.match_indices("{# elseif") {
        let Some(elif_end) = tmpl[i..].find("#}") else {
            continue;
        };
        let cond = tmpl[i + 9..elif_end].trim();
        let up_to = tmpl[elif_end+2..].find("{# elseif");
        let block = parse_tmpl(tmpl[elif_end+2..up_to])
    }

    // Find else
    if let Some(else_start) = tmpl.find("{# else #}") {
        // parse block from here to overall end
    }

    None
}

fn find_first(haystack: &str, patterns: &'static [&'static str]) -> Option<usize> {

}

fn fill_tmpl<'facet, 'mem, Ctx: Facet<'facet> + ?Sized>(
    chunks: &[Chunk<'_>],
    ctx: &'mem Ctx,
) -> Result<String, TemplateError> {
    let mut ret = String::new();
    let peek = deref_refs(Peek::new(ctx));

    let ctx_struct = peek.into_struct().map_err(|e| TemplateError {
        kind: TemplateErrorKind::InvalidContextError { reflect_error: e },
    })?; // InvalidContext
    for chunk in chunks {
        match chunk {
            Chunk::Text(text) => ret.push_str(text),
            Chunk::Expr(ident) => {
                let field = ctx_struct
                    .field_by_name(ident)
                    .map_err(|_e| TemplateError {
                        kind: TemplateErrorKind::MissingFieldError {
                            expected: (*ident).to_owned(),
                        },
                    })?;
                ret.push_str(&format!("{}", field));
            }
        }
    }
    Ok(ret)
}

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
        let template = "Hello {name}!";
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

        let template = "Hello {name}!";
        let chunks = parse_tmpl(template);
        // Note: the && is on purpose to check we handle ref-following correctly
        let ret = fill_tmpl(&chunks, &&ctx).unwrap();
        assert_eq!(ret, "Hello Vader!");
    }
}
