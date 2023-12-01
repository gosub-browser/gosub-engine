use crate::css3::ast::Node;
use nom::branch::alt;
use nom::combinator::{map, opt};
use nom::multi::{many0, separated_list1};
use nom::IResult;
use crate::css3::nom::{any_function, any_ident, comma, delim, ident, whitespace0, whitespace1};
use crate::css3::span::Span;

/// This module contains functions to parse Media queries. For more information see:
/// https://www.w3.org/TR/mediaqueries-4/

pub(crate) fn parse_media_query_list(input: Span) -> IResult<Span, Node> {
    let (input, media_queries) = separated_list1(comma, parse_media_query)(input)?;

    let mut node = Node::new("MediaQueryList");
    node.children = media_queries;

    Ok((input, node))
}

// <media-query> = <media-condition> | [ not | only ]? <media-type> [ and <media-condition-without-or> ]?
fn parse_media_query(input: Span) -> IResult<Span, Node> {
    let (input, media_query) = alt((
        |i| parse_media_condition(i),
        |i| {
            let (i, _) = opt(|i| ident(i, "not".to_string()))(i)?;
            let (i, _) = whitespace0(i)?;
            let (i, _) = opt(|i| ident(i, "only".to_string()))(i)?;
            let (i, _) = whitespace0(i)?;
            let (i, media_type) = parse_media_type(i)?;
            let (i, _) = whitespace0(i)?;
            let (i, _) = opt(|i| ident(i, "and".to_string()))(i)?;
            let (i, _) = whitespace0(i)?;
            let (i, media_condition_without_or) = opt(|i| parse_media_without_or(i))(i)?;

            let mut node = Node::new("MediaQuery");
            node.children.push(media_type);

            if let Some(media_condition_without_or) = media_condition_without_or {
                node.children.push(media_condition_without_or);
            }

            Ok((i, node))
        }
    ))(input)?;

    Ok((input, media_query))
}

// <media-type> = <ident> (except for "only", "not" "and" "or"
fn parse_media_type(input: Span) -> IResult<Span, Node> {
    let (input, ident) = any_ident(input)?;

    if ident == "only" || ident == "not" || ident == "and" || ident == "or" {
        return Err(nom::Err::Error(nom::error::Error::new(
            input.clone(),
            nom::error::ErrorKind::Tag,
        )))
    }

    let mut node = Node::new("MediaType");
    node.attributes.insert("name".to_string(), ident);

    Ok((input, node))
}

// <media-condition> = <media-not> | <media-in-parens> [ <media-and>* | <media-or>* ]
fn parse_media_condition(input: Span) -> IResult<Span, Node> {
    // Must have either a media-not, or media-in-parens
    let (input, mut media_condition) = alt((
        |i| parse_media_not(i),
        |i| parse_media_in_parens(i),
    ))(input)?;

    // Must have zero or more media-ands, OR zero or more media-ors
    let (input, cond) = alt((
        |i|
            many0(|i| parse_media_and(i))(i),
        |i| many0(|i| parse_media_and(i))(i),
    ))(input)?;

    // add condition (media-and* | media-or*) as child onder media-condition (media-not | media-in-parens)
    media_condition.children = cond;

    let mut node = Node::new("MediaCondition");
    node.children.push(media_condition);

    Ok((input, node))
}

// <media-condition-without-or> = <media-not> | <media-in-parens> <media-and>*
fn parse_media_without_or(input: Span) -> IResult<Span, Node> {
    let (input, _media_condition_without_or) = alt((
        |i| parse_media_not(i),
        |i| parse_media_in_parens(i),
    ))(input)?;

    let (input, ands) = many0(|i| parse_media_and(i))(input)?;
    let mut node = Node::new("MediaConditionWithoutOr");
    node.children = ands;

    Ok((input, node))
}

// <media-not> = not <media-in-parens>
fn parse_media_not(input: Span) -> IResult<Span, Node> {
    let (input, _) = ident(input, "not".to_string())?;
    let (input, _) = whitespace1(input)?;
    let (input, media_in_parens) = parse_media_in_parens(input)?;

    let mut node = Node::new("MediaNot");
    node.children.push(media_in_parens);

    Ok((input, node))

}

// <media-and> = and <media-in-parens>
fn parse_media_and(input: Span) -> IResult<Span, Node> {
    let (input, _) = ident(input, "and".to_string())?;
    let (input, _) = whitespace1(input)?;
    let (input, media_in_parens) = parse_media_in_parens(input)?;

    let mut node = Node::new("MediaAnd");
    node.children.push(media_in_parens);

    Ok((input, node))

}

// <media-or> = or <media-in-parens>
fn parse_media_or(input: Span) -> IResult<Span, Node> {
    let (input, _) = ident(input, "or".to_string())?;
    let (input, _) = whitespace1(input)?;
    let (input, media_in_parens) = parse_media_in_parens(input)?;

    let mut node = Node::new("MediaOr");
    node.children.push(media_in_parens);

    Ok((input, node))
}

// <media-in-parens> = ( <media-condition> ) | ( <media-feature> ) | <general-enclosed>
fn parse_media_in_parens(input: Span) -> IResult<Span, Node> {
    let (input, media_in_parens) = alt((
        |i| {
            let (i, _) = delim(i, '(')?;
            let (i, condition) = parse_media_condition(i)?;
            let (i, _) = delim(i, ')')?;

            Ok((i, condition))
        },
        |i| {
            let (i, _) = delim(i, '(')?;
            let (i, feature) = parse_media_feature(i)?;
            let (i, _) = delim(i, ')')?;

            Ok((i, feature))
        },
        |i| general_enclosed(i),
    ))(input)?;

    let mut node = Node::new("MediaInParens");
    node.children.push(media_in_parens);

    Ok((input, node))
}

// <media-feature> = [ <mf-plain> | <mf-boolean> | <mf-range> ]
fn parse_media_feature(input: Span) -> IResult<Span, Node> {
    let (input, _) = delim(input, '[')?;
    let (input, feature) = alt((
        |i| mf_plain(i),
        |i| mf_plain(i),
        |i| mf_boolean(i),
        |i| mf_range(i),
    ))(input)?;
    let (input, _) = delim(input, ']')?;

    let mut node = Node::new("MediaFeature");
    node.children.push(feature);

    Ok((input, node))
}

// <mf-plain> = <mf-name> : <mf-value>
fn mf_plain(input: Span) -> IResult<Span, Node> {
    let (input, name) = mf_name(input)?;
    let (input, _) = delim(input, ':')?;
    let (input, _) = whitespace1(input)?;
    let (input, value) = mf_value(input)?;
    let (input, _) = whitespace1(input)?;

    let mut node = Node::new("PlainMediaFeature");
    node.attributes.insert("name".to_string(), name);
    node.children.push(value);

    Ok((input, node))
}

// <mf-boolean> = <mf-name>
fn mf_boolean(input: Span) -> IResult<Span, Node> {
    let (input, name) = mf_name(input)?;

    let mut node = Node::new("BooleanMediaFeature");
    node.attributes.insert("name".to_string(), name);

    Ok((input, node))
}

// <mf-range> = <mf-name> <mf-comparison> <mf-value> | <mf-value> <mf-comparison> <mf-name> | <mf-value> <mf-lt> <mf-name> <mf-lt> <mf-value> | <mf-value> <mf-gt> <mf-name> <mf-gt> <mf-value>
fn mf_range(input: Span) -> IResult<Span, Node> {
    let (input, range) = alt((
        |i| {
            let (i, name) = mf_name(i)?;
            let (i, comparison) = mf_comparison(i)?;
            let (i, value) = mf_value(i)?;

            let mut node = Node::new("RangeMediaFeature");
            node.attributes.insert("name".to_string(), name);
            node.attributes.insert("comparison".to_string(), comparison);
            node.children.push(value);

            Ok((i, node))
        },
        |i| {
            let (i, value) = mf_value(i)?;
            let (i, comparison) = mf_comparison(i)?;
            let (i, name) = mf_name(i)?;

            let mut node = Node::new("RangeMediaFeature");
            node.attributes.insert("name".to_string(), name);
            node.attributes.insert("comparison".to_string(), comparison);
            node.children.push(value);

            Ok((i, node))
        },
        |i| {
            let (i, value) = mf_value(i)?;
            let (i, lt) = mf_lt(i)?;
            let (i, name) = mf_name(i)?;
            let (i, gt) = mf_gt(i)?;
            let (i, value2) = mf_value(i)?;

            let mut node = Node::new("RangeMediaFeature");
            node.attributes.insert("name".to_string(), name);
            node.attributes.insert("lt".to_string(), lt);
            node.attributes.insert("gt".to_string(), gt);
            node.children.push(value);
            node.children.push(value2);

            Ok((i, node))
        },
        |i| {
            let (i, value) = mf_value(i)?;
            let (i, gt) = mf_gt(i)?;
            let (i, name) = mf_name(i)?;
            let (i, lt) = mf_lt(i)?;
            let (i, value2) = mf_value(i)?;

            let mut node = Node::new("RangeMediaFeature");
            node.attributes.insert("name".to_string(), name);
            node.attributes.insert("lt".to_string(), lt);
            node.attributes.insert("gt".to_string(), gt);
            node.children.push(value);
            node.children.push(value2);

            Ok((i, node))
        },
    ))(input)?;

    Ok((input, range))
}

// <mf-name> = <ident>
fn mf_name(input: Span) -> IResult<Span, String> {
    any_ident(input)
}

// <mf-value> = <number> | <dimension> | <ident> | <ratio>
fn mf_value(input: Span) -> IResult<Span, Node> {
    alt((
        |i| number(i),
        |i| dimension(i),
        |i| map(|i| any_ident(i), |name| Node::new("Ident").with_attribute("name", name))(i),
        |i| ratio(i),
    ))(input)
}

// <mf-lt> = '<' '='?
fn mf_lt(input: Span) -> IResult<Span, String> {
    let (input, span) = delim(input, '<')?;
    let (input, _) = whitespace0(input)?;
    let (input, _) = opt(|i| delim(i, '='))(input)?;
    let (input, _) = whitespace0(input)?;

    Ok((input, span.to_string()))
}

// <mf-gt> = '>' '='?
fn mf_gt(input: Span) -> IResult<Span, String> {
    let (input, span) = delim(input, '>')?;
    let (input, _) = whitespace0(input)?;
    let (input, _) = opt(|i| delim(i, '='))(input)?;
    let (input, _) = whitespace0(input)?;

    Ok((input, span.to_string()))
}

// <mf-eq> = '='
fn mf_eq(input: Span) -> IResult<Span, String> {
    let (input, span) = delim(input, '=')?;
    let (input, _) = whitespace0(input)?;

    Ok((input, span.to_string()))
}

// <mf-comparison> = <mf-lt> | <mf-gt> | <mf-eq>
fn mf_comparison(input: Span) -> IResult<Span, String> {
    alt((
        mf_lt,
        mf_gt,
        mf_eq,
    ))(input)
}

// <general-enclosed> = [ <function-token> <any-value>? ) ] | ( <any-value>? )
fn general_enclosed(input: Span) -> IResult<Span, Node> {
    let mut node = Node::new("GeneralEnclosed");

    let (input, node) = alt((
        |i| {
            let (i, _) = delim(i, '(')?;
            let (i, function_token) = opt(|i| any_function(i))(i)?;
            let (i, any_value) = opt(|i| any_ident(i))(i)?;
            let (i, _) = delim(i, ')')?;

            if function_token.is_some() {
                node.attributes.insert("function".to_string(), function_token.unwrap());
            }
            if any_value.is_some() {
                node.attributes.insert("value".to_string(), any_value.unwrap());
            }

            Ok((i, node.clone()))
        },
        |i| {
            let (i, _) = delim(i, '(')?;
            let (i, any_value) = opt(|i| any_ident(i))(i)?;
            let (i, _) = delim(i, ')')?;

            if any_value.is_some() {
                node.attributes.insert("value".to_string(), any_value.unwrap());
            }

            Ok((i, node.clone()))
        }
    ))(input)?;

    // function( any-value )
    // function( )
    // ( any-value )
    // ( )

    Ok((input, node))
}

fn number(_input: Span) -> IResult<Span, Node> {
    todo!()
}

fn dimension(_input: Span) -> IResult<Span, Node> {
    todo!()
}

fn ratio(_input: Span) -> IResult<Span, Node> {
    todo!()
}
