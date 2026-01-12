use nom::multi::many0;

use super::parser::{parse_arbitrary_attribute_variant, parse_data_attribute_variant, parse_variant};
use super::{ASTVariant, AstParseOptions, AstStyle};

fn parse_tailwind<'a>(class: &'a str) -> Vec<Result<AstStyle<'a>, &'a str>> {
    let options = AstParseOptions::default();
    let split = class.split_whitespace().collect::<Vec<_>>();
    super::parse_tailwind(split.as_slice(), options)
}

#[test]
fn basic_parse() {
    let class = "flex justify-between items-center";
    let result = parse_tailwind(class);
    let expected = vec![
        Ok(AstStyle {
            source: "flex",
            important: false,
            negative: false,
            variants: vec![],
            elements: vec!["flex"],
            arbitrary: None,
        }),
        Ok(AstStyle {
            source: "justify-between",
            important: false,
            negative: false,
            variants: vec![],
            elements: vec!["justify", "between"],
            arbitrary: None,
        }),
        Ok(AstStyle {
            source: "items-center",
            important: false,
            negative: false,
            variants: vec![],
            elements: vec!["items", "center"],
            arbitrary: None,
        }),
    ];

    assert_eq!(result, expected)
}

#[test]
fn test_with_options() {
    let class = "dark|hover|tw-flex";
    let class = [class];
    let options = AstParseOptions { prefix: "tw-", separator: "|" };
    let result = super::parse_tailwind(&class, options);
    let expected = vec![Ok(AstStyle {
        source: "dark|hover|tw-flex",
        important: false,
        negative: false,
        variants: vec!["dark", "hover"],
        elements: vec!["flex"],
        arbitrary: None,
    })];

    assert_eq!(result, expected)
}

#[test]
fn parse_with_negative() {
    let class = "-my-2";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "-my-2",
        important: false,
        negative: true,
        variants: vec![],
        elements: vec!["my", "2"],
        arbitrary: None,
    })];
    assert_eq!(result, expected)
}

#[test]
fn test_with_important() {
    let class = "!bg-blue-500";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "!bg-blue-500",
        important: true,
        negative: false,
        variants: vec![],
        elements: vec!["bg", "blue", "500"],
        arbitrary: None,
    })];
    assert_eq!(result, expected)
}

// v4: important modifier at end of class
#[test]
fn test_with_important_v4_suffix() {
    let class = "bg-blue-500!";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "bg-blue-500!",
        important: true,
        negative: false,
        variants: vec![],
        elements: vec!["bg", "blue", "500"],
        arbitrary: None,
    })];
    assert_eq!(result, expected)
}

#[test]
fn multiple_variants() {
    let class = "hover:md:flex";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "hover:md:flex",
        important: false,
        negative: false,
        variants: vec!["hover", "md"],
        elements: vec!["flex"],
        arbitrary: None,
    })];
    assert_eq!(result, expected)
}

#[test]
fn aria_attributes() {
    let class = "aria-checked:true";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "aria-checked:true",
        important: false,
        negative: false,
        variants: vec!["aria-checked"],
        elements: vec!["true"],
        arbitrary: None,
    })];
    assert_eq!(result, expected)
}

#[test]
fn arbitrary_variants() {
    let class = "[&:nth-child(3)]:underline";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "[&:nth-child(3)]:underline",
        important: false,
        negative: false,
        variants: vec!["[&:nth-child(3)]"],
        elements: vec!["underline"],
        arbitrary: None,
    })];
    assert_eq!(result, expected)
}

#[test]
fn test_data_attribute() {
    let (rest, variant) = parse_data_attribute_variant("data-[open]:flex-col").unwrap();
    assert_eq!(":flex-col", rest);
    assert_eq!(ASTVariant::DataAttribute("data-[open]"), variant);

    let class = "data-[open]:flex-col data-[close]:flex-row";
    let result = parse_tailwind(class);

    let expected = vec![
        Ok(AstStyle {
            source: "data-[open]:flex-col",
            important: false,
            negative: false,
            variants: vec!["data-[open]"],
            elements: vec!["flex", "col"],
            arbitrary: None,
        }),
        Ok(AstStyle {
            source: "data-[close]:flex-row",
            important: false,
            negative: false,
            variants: vec!["data-[close]"],
            elements: vec!["flex", "row"],
            arbitrary: None,
        }),
    ];

    assert_eq!(result, expected)
}

#[test]
fn test_variants() {
    let class = "dark:lg:hover:[&>*]:line-through";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "dark:lg:hover:[&>*]:line-through",
        important: false,
        negative: false,
        variants: vec!["dark", "lg", "hover", "[&>*]"],
        elements: vec!["line", "through"],
        arbitrary: None,
    })];
    assert_eq!(result, expected);
}

#[test]
fn test_arbitrary_variant_parse() {
    let class = "dark:lg:hover:[&>*]:line-through";
    let mut parser = many0(|s| parse_variant(":", s));
    let (str, variant) = parser(class).unwrap();

    assert_eq!(str, "line-through");
    let expected = vec![
        ASTVariant::Normal("dark"),
        ASTVariant::Normal("lg"),
        ASTVariant::Normal("hover"),
        ASTVariant::ArbitraryAttribute("[&>*]"),
    ];
    assert_eq!(variant, expected)
}

#[test]
fn test_take_until_unbalanced() {
    let input = "[&:nth-child(3)]:underline";
    let (rest, result) = parse_arbitrary_attribute_variant(input).unwrap();
    assert_eq!(rest, ":underline");
    assert_eq!(result, ASTVariant::ArbitraryAttribute("[&:nth-child(3)]"));
}

#[test]
fn test_nested_variants() {
    let class = "[&[data-open]]:line-through";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "[&[data-open]]:line-through",
        important: false,
        negative: false,
        variants: vec!["[&[data-open]]"],
        elements: vec!["line", "through"],
        arbitrary: None,
    })];

    assert_eq!(result, expected);
}

#[test]
fn non_tailwind() {
    let class = "data-[key=value flex";
    let result = parse_tailwind(class);
    let expected = vec![
        Err("data-[key=value"),
        Ok(AstStyle {
            source: "flex",
            important: false,
            negative: false,
            variants: vec![],
            elements: vec!["flex"],
            arbitrary: None,
        }),
    ];
    assert_eq!(result, expected)
}

#[test]
fn test_double_arbitrary() {
    let class = "[&>*]:[color:blue]";
    let result = parse_tailwind(class);
    let expected = vec![Ok(AstStyle {
        source: "[&>*]:[color:blue]",
        important: false,
        negative: false,
        variants: vec!["[&>*]"],
        elements: vec![],
        arbitrary: Some("color:blue"),
    })];
    assert_eq!(result, expected)
}
