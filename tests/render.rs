use facet::Facet;
use facet_template::Template;

fn check<'facet, Ctx: Facet<'facet>>(tmpl: &'static str, ctx: Ctx, expected: &'static str) {
    let found = Template::new(tmpl).fill(&ctx).unwrap();
    assert_eq!(found, expected);
}

fn s(st: &'static str) -> String {
    String::from(st)
}

#[test]
fn test_basic_replace() {
    #[derive(Facet)]
    struct Ctx {
        name: String,
    }

    check(
        "Hello, my name is {name}.",
        Ctx { name: s("Seth") },
        "Hello, my name is Seth.",
    )
}

#[test]
fn test_replace() {
    #[derive(Facet)]
    struct Ctx {
        name: String,
        id: usize,
    }

    check(
        "Hello, my name is {name}. The id is {id}.",
        Ctx {
            name: s("Seth"),
            id: 42,
        },
        "Hello, my name is Seth. The id is 42.",
    )
}

#[test]
fn test_replace_custom_object() {
    #[derive(Facet)]
    struct Custom {
        special_field: f64,
        special_name: String,
    }

    impl core::fmt::Display for Custom {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(
                f,
                "{{special_field: {}, special_name: {}}}",
                self.special_field, self.special_name
            )
        }
    }

    #[derive(Facet)]
    struct Ctx {
        name: String,
        custom: Custom,
        multi: Vec<usize>,
    }

    check(
        "Hello, my name is {name}. Custom profile is {custom}.",
        Ctx {
            name: s("Seth"),
            custom: Custom {
                special_field: 42.1,
                special_name: s("Also Seth"),
            },
            multi: vec![0, 1, 2, 3],
        },
        "Hello, my name is Seth. Custom profile is {special_field: 42.1, special_name: Also Seth.}.",
    )
}

#[test]
fn test_replace() {
    #[derive(Facet)]
    struct Ctx {
        name: String,
        id: usize,
    }

    check(
        "Hello, my name is {name}. The id is {id}.
{#if foo}Foo is True{#endif}
{#if !foo}Foo is not True{#else}Foo is True{#endif}
{#if !foo}Foo is notTrue{#elseif bar}Bar is True{#elseif car}Car is True{#endif}
",
        Ctx {
            name: s("Seth"),
            id: 42,
        },
        "Hello, my name is Seth. The id is 42.",
    )
}
