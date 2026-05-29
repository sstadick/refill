# refill

[![Crates.io](https://img.shields.io/crates/v/refill.svg)](https://crates.io/crates/refill)
[![Docs.rs](https://docs.rs/refill/badge.svg)](https://docs.rs/refill)
[![License](https://img.shields.io/crates/l/refill.svg)](https://crates.io/crates/refill)

A very small template library based on [`facet`](https://github.com/facet-rs/facet)

## Example

```rust
use facet::Facet;
use refill::Template;

#[derive(Facet)]
struct Custom {
    special_field: f64,
    special_name: String,
}

#[derive(Facet)]
struct Ctx<'a> {
    name: String,
    custom: &'a Custom,
    multi: Vec<usize>,
}

fn main() {
    let c = Custom {
        special_field: 42.1,
        special_name: String::from("Also Seth"),
    };
    let ctx = Ctx {
        name: String::from("Seth"),
        custom: &c,
        multi: vec![0, 1, 2, 3],
    };

    let found = Template::new("Hello, my name is {{custom.special_name}}.").fill(&ctx).unwrap();
    let expected = "Hello, my name is Also Seth.";
    assert_eq!(found, expected);
}
```

## Future Features

This may grow more to include expressions, but I have not needed them yet.
