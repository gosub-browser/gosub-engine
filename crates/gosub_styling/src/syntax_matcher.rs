use crate::syntax::{GroupCombinators, SyntaxComponent, SyntaxComponentMultiplier};
use gosub_css3::colors::{is_named_color, is_system_color};
use gosub_css3::stylesheet::CssValue;

/// Structure to return from a matching function.
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// The remainder of the values that are not matched.
    pub remainder: Vec<CssValue>,
    /// True when this matched did some matching (todo: we might remove this and check for matched_values.is_empty)
    pub matched: bool,
    /// List of the matched values
    pub matched_values: Vec<CssValue>,
}

#[allow(dead_code)]
const LENGTH_UNITS: [&str; 31] = [
    "cap", "ch", "em", "ex", "ic", "lh", "rcap", "rch", "rem", "rex", "ric", "rlh", "vh", "vw",
    "vmax", "vmin", "vb", "vi", "cqw", "cqh", "cqi", "cqb", "cqmin", "cqmax", "px", "cm", "mm",
    "Q", "in", "pc", "pt",
];

/// A CSS Syntax Tree is a tree sof CSS syntax components that can be used to match against CSS values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSyntaxTree {
    /// The components of the syntax tree
    pub components: Vec<SyntaxComponent>,
}

impl CssSyntaxTree {
    /// Creates a new CSS Syntax tree from the given components
    pub fn new(components: Vec<SyntaxComponent>) -> Self {
        CssSyntaxTree { components }
    }

    /// Matches a CSS value (or set of values) against the syntax tree. Will return a normalized version of the value(s) if it matches.
    pub fn matches(&self, input: Vec<CssValue>) -> bool {
        if self.components.len() != 1 {
            panic!("Syntax tree must have exactly one root component");
        }

        let res = match_component(&input, &self.components[0]);
        res.matched && res.remainder.is_empty()
    }
}

fn match_component_inner(raw_input: &Vec<CssValue>, component: &SyntaxComponent) -> MatchResult {
    let _gid = rand::random::<u8>();

    let mut input = raw_input.clone();
    let mut matched_values = vec![];

    // Loop through the input values and try to match them against the component. It's possible
    // that we need to loop multiple times in case we have a multiplier that allows this. (ie: 'foo*' or 'foo{1,3}')
    let mut multiplier_count = 0;
    loop {
        if input.is_empty() {
            // We don't have anything in the input stream. We do need to check if this component
            // allows for optional values. If so, the component matches.
            let mff = multiplier_fulfilled(component, 0);
            if mff == Fulfillment::Fulfilled || mff == Fulfillment::FulfilledButMoreAllowed {
                return MatchResult {
                    remainder: vec![],
                    matched: true,
                    matched_values: vec![],
                };
            }

            // Seems this component needs at least one value. We don't have any, so it's no match
            return no_match(raw_input);
        }

        // Check either single or group component
        let res = if component.is_group() {
            match_component_group(&input, component)
        } else {
            match_component_single(&input, component)
        };

        if res.matched {
            // The element matched, so we keep track on how many times it did (in case of multipliers)
            multiplier_count += 1;

            matched_values.append(&mut res.matched_values.clone());

            // Check if we fulfilled the multiplier for this component
            let mff = multiplier_fulfilled(component, multiplier_count);
            match mff {
                Fulfillment::NotYetFulfilled => {
                    // The multiplier is not yet fulfilled. Probably a range multiplier, so we need more
                    // values. Loop to the next value.
                    input = res.remainder.clone();
                    continue;
                }
                Fulfillment::FulfilledButMoreAllowed => {
                    // More elements are allowed. Let's check if we have one
                    input = res.remainder.clone();

                    // No more input to check, so we can just return this match
                    if input.is_empty() {
                        return res;
                    }
                }
                Fulfillment::Fulfilled => {
                    // no more values are allowed.
                    return res;
                }
                Fulfillment::NotFulfilled => {
                    // The multiplier is not fulfilled.
                    return no_match(raw_input);
                }
            }
        } else {
            let mff = multiplier_fulfilled(component, multiplier_count);
            match mff {
                Fulfillment::NotYetFulfilled => {
                    // Don't know about this case
                    return res;
                }
                Fulfillment::Fulfilled => {
                    return res;
                }
                Fulfillment::FulfilledButMoreAllowed => {
                    return MatchResult {
                        remainder: input.clone(),
                        matched: true,
                        matched_values,
                    };
                }
                Fulfillment::NotFulfilled => {
                    return no_match(raw_input);
                }
            }
        }
    }
}

/// Matches a component against the input values. After the match, there might be remaining
/// elements in the input. This is passed back in the MatchResult structure.
fn match_component(raw_input: &Vec<CssValue>, component: &SyntaxComponent) -> MatchResult {
    let _gid = rand::random::<u8>();

    let mut input = raw_input.clone();

    // Set some additional values when we are dealing with a comma separated lists (the # multiplier)
    let mut comma_separated = false;
    let mut csv_cnt = 0;
    let mut csv_min = 0;
    let mut csv_max = 0;
    let mut matched_values = vec![];
    for multiplier in component.get_multipliers() {
        match multiplier {
            SyntaxComponentMultiplier::CommaSeparatedRepeat(min, max) => {
                comma_separated = true;
                csv_min = min;
                csv_max = max;
            }
            _ => {}
        }
    }

    // CSV loop
    loop {
        let inner_result = match_component_inner(&input, component);
        if !comma_separated {
            // We don't need to check for comma separated values, so just return this result
            return inner_result;
        }

        if !inner_result.matched {
            // Not matched, so break the loop
            break;
        }

        csv_cnt += 1;
        matched_values.append(&mut inner_result.matched_values.clone());

        input = inner_result.remainder.clone();

        // It's the end, do doo doo doo.
        if input.is_empty() {
            break;
        }

        // Make sure the next value from input is a comma
        if input.first() != Some(&CssValue::Comma) {
            return no_match(raw_input);
        }

        // Remove the comma, and continue matching
        input = input[1..input.len()].to_owned();

        if input.is_empty() {
            // We have a comma at the end of the input. This is not allowed.
            return no_match(raw_input);
        }
    }

    // If we are in a comma separated list, we need to check if we have the correct amount of values
    if comma_separated && csv_cnt >= csv_min && csv_cnt <= csv_max {
        return MatchResult {
            remainder: input.clone(),
            matched: true,
            matched_values,
        };
    }

    no_match(raw_input)
}

/// Matches a component group
fn match_component_group(input: &Vec<CssValue>, component: &SyntaxComponent) -> MatchResult {
    match &component {
        SyntaxComponent::Group {
            components,
            combinator,
            ..
        } => {
            // println!("We need to do a group match on {:?}, our value is: {:?}", combinator, input);

            

            match combinator {
                GroupCombinators::Juxtaposition => match_group_juxtaposition(input, components),
                GroupCombinators::AllAnyOrder => match_group_all_any_order(input, components),
                GroupCombinators::AtLeastOneAnyOrder => {
                    match_group_at_least_one_any_order(input, components)
                }
                GroupCombinators::ExactlyOne => match_group_exactly_one(input, components),
            }
        }
        e => {
            panic!("Unknown syntax component group: {:?}", e);
        }
    }
}

/// Matches a single component value
fn match_component_single(input: &Vec<CssValue>, component: &SyntaxComponent) -> MatchResult {
    // Get the first value from the input which we will use for matching
    let value = input.first().unwrap();

    // println!("\n\nmatch_component: {:?} against {:?}", value, component);

    match &component {
        SyntaxComponent::GenericKeyword { keyword, .. } => match value {
            CssValue::None if keyword.eq_ignore_ascii_case("none") => {
                return first_match(input);
            }
            CssValue::String(v) if v.eq_ignore_ascii_case(keyword) => {
                // println!("keyword {:?} match!", v);
                return first_match(input);
            }
            _ => {}
        },
        SyntaxComponent::Definition { .. } => {
            //dbg!(&component);
            todo!("Definition not implemented yet");
        }
        SyntaxComponent::Builtin { datatype, .. } => match datatype.as_str() {
            "percentage" => match value {
                CssValue::Percentage(_) => return first_match(input),
                _ => {}
            },
            "angle" => match value {
                CssValue::Zero => return first_match(input),
                CssValue::Unit(_, u) if u.eq_ignore_ascii_case("deg") => return first_match(input),
                CssValue::Unit(_, u) if u.eq_ignore_ascii_case("grad") => {
                    return first_match(input)
                }
                CssValue::Unit(_, u) if u.eq_ignore_ascii_case("rad") => return first_match(input),
                CssValue::Unit(_, u) if u.eq_ignore_ascii_case("turn") => {
                    return first_match(input)
                }
                _ => {}
            },
            "length" => match value {
                CssValue::Zero => return first_match(input),
                CssValue::Unit(_, u) if LENGTH_UNITS.contains(&u.as_str()) => {
                    return first_match(input)
                }
                _ => {}
            },
            "system-color" => match value {
                CssValue::String(v) => {
                    if is_system_color(v) {
                        return first_match(input);
                    }
                }
                _ => {}
            },
            "named-color" => match value {
                CssValue::String(v) => {
                    if is_named_color(v) {
                        return first_match(input);
                    }
                }
                _ => {}
            },
            "color()" => match value {
                // @TODO: fix this according to what the spec says
                CssValue::Color(_) => return first_match(input),
                CssValue::String(v) if v.starts_with('#') => return first_match(input),
                _ => {}
            },
            "hex-color" => match value {
                CssValue::Color(_) => return first_match(input),
                CssValue::String(v) if v.starts_with('#') => return first_match(input),
                _ => {}
            },
            _ => panic!("Unknown built-in datatype: {:?}", datatype),
        },
        SyntaxComponent::Inherit { .. } => match value {
            CssValue::Inherit => return first_match(input),
            CssValue::String(v) if v.eq_ignore_ascii_case("inherit") => return first_match(input),
            _ => {}
        },
        SyntaxComponent::Initial { .. } => match value {
            CssValue::Initial => return first_match(input),
            CssValue::String(v) if v.eq_ignore_ascii_case("initial") => return first_match(input),
            _ => {}
        },
        SyntaxComponent::Unset { .. } => match value {
            CssValue::String(v) if v.eq_ignore_ascii_case("unset") => return first_match(input),
            _ => {}
        },
        SyntaxComponent::Unit { from, to, unit, .. } => {
            let f32min = f32::MIN;
            let f32max = f32::MAX;

            match value {
                CssValue::Number(n) if *n == 0.0 => return first_match(input),
                CssValue::Unit(n, u) => {
                    if unit.contains(u)
                        && *n >= from.unwrap_or(f32min)
                        && *n <= to.unwrap_or(f32max)
                    {
                        return first_match(input);
                    }
                }
                _ => {}
            };
        }
        SyntaxComponent::Literal { literal, .. } => match value {
            CssValue::String(v) if v.eq(literal) => return first_match(input),
            CssValue::String(v) if v.eq_ignore_ascii_case(literal) => {
                log::warn!("Case insensitive literal matched");
                return first_match(input);
            }
            _ => {}
        },
        SyntaxComponent::Function { name, .. } => {
            let CssValue::Function(c_name, c_args) = value else {
                return no_match(input);
            };

            if !name.eq_ignore_ascii_case(c_name) {
                return no_match(input);
            }

            if c_args.is_empty() {
                return first_match(input);
            }

            todo!("Function not implemented yet. We must match the arguments");
            // let list = CssValue::List(c_args.clone());
            // return match_internal(&list, arguments);
        }
        SyntaxComponent::Value {
            value: css_value, ..
        } => {
            if value == css_value {
                return first_match(input);
            }
        }
        e => {
            panic!("Unknown syntax component: {:?}", e);
        }
    }

    no_match(input)
}

/// Returns element if exactly one element matches in the group
fn match_group_exactly_one(
    raw_input: &Vec<CssValue>,
    components: &Vec<SyntaxComponent>,
) -> MatchResult {
    let _gid = rand::random::<u8>();

    let input = raw_input.to_vec();
    let mut matched_values = vec![];
    let mut components_matched = vec![];

    let mut c_idx = 0;
    while c_idx < components.len() {
        if input.is_empty() {
            break;
        }

        let component = &components[c_idx];

        let res = match_component(&input, component);
        if res.matched {
            matched_values.append(&mut res.matched_values.clone());

            // input = res.remainder.clone();

            components_matched.push((c_idx, res.matched_values, res.remainder));
        } else {
            // No match. That's all right.
        }

        c_idx += 1;
    }

    if components_matched.is_empty() {
        return no_match(&input);
    }

    if components_matched.len() > 1 {
        let mut shortest_remainder_idx = 0;
        let mut shortest_remainder_len = components_matched.first().unwrap().2.len();

        for (idx, (_, _, remainder)) in components_matched.iter().enumerate() {
            if remainder.len() < shortest_remainder_len {
                shortest_remainder_len = remainder.len();
                shortest_remainder_idx = idx;
            }
        }

        return MatchResult {
            remainder: components_matched[shortest_remainder_idx].2.clone(),
            matched: true,
            matched_values: components_matched[shortest_remainder_idx].1.clone(),
        };
    }

    MatchResult {
        remainder: components_matched[0].2.clone(),
        matched: true,
        matched_values: components_matched[0].1.clone(),
    }
}

/// Returns element, when at least one of the elements in the group matches
fn match_group_at_least_one_any_order(
    raw_input: &Vec<CssValue>,
    components: &Vec<SyntaxComponent>,
) -> MatchResult {
    let mut input = raw_input.to_vec();
    let mut matched_values = vec![];
    let mut components_matched = vec![];

    let mut c_idx = 0;
    while c_idx < components.len() {
        if input.is_empty() {
            break;
        }

        let component = &components[c_idx];

        let res = match_component(&input, component);
        if res.matched {
            matched_values.append(&mut res.matched_values.clone());
            components_matched.push(c_idx);

            input = res.remainder.clone();

            // Found a match, so loop around for new matches
            c_idx = 0;
            while components_matched.contains(&c_idx) {
                c_idx += 1;
            }
        } else {
            // Element didn't match. That might be allright, and we continue with the next unmatched component
            c_idx += 1;
            while components_matched.contains(&c_idx) {
                c_idx += 1;
            }
        }
    }

    if components_matched.is_empty() {
        return no_match(&input);
    }

    MatchResult {
        remainder: input.clone(),
        matched: true,
        matched_values,
    }
}

fn match_group_all_any_order(
    raw_input: &Vec<CssValue>,
    components: &Vec<SyntaxComponent>,
) -> MatchResult {
    let mut input = raw_input.to_vec();
    let mut matched_values = vec![];
    let mut components_matched = vec![];

    let mut c_idx = 0;
    while c_idx < components.len() {
        if input.is_empty() {
            break;
        }

        let component = &components[c_idx];

        let res = match_component(&input, component);
        if res.matched {
            matched_values.append(&mut res.matched_values.clone());
            components_matched.push(c_idx);

            input = res.remainder.clone();

            // Found a match, so loop around for new matches
            c_idx = 0;
            while components_matched.contains(&c_idx) {
                c_idx += 1;
            }
        } else {
            // Element didn't match. That might be allright, and we continue with the next unmatched component
            c_idx += 1;
            while components_matched.contains(&c_idx) {
                c_idx += 1;
            }
        }
    }

    if components_matched.len() != components.len() {
        return no_match(&input);
    }

    MatchResult {
        remainder: input.clone(),
        matched: true,
        matched_values,
    }
}

fn match_group_juxtaposition(
    raw_input: &Vec<CssValue>,
    components: &Vec<SyntaxComponent>,
) -> MatchResult {
    let mut input = raw_input.to_vec();
    let mut matched_values = vec![];

    let mut c_idx = 0;
    while c_idx < components.len() {
        let component = &components[c_idx];

        let res = match_component(&input, component);
        if res.matched {
            matched_values.append(&mut res.matched_values.clone());
            input = res.remainder.clone();
        } else {
            break;
        }

        c_idx += 1;
    }

    if c_idx != components.len() {
        return no_match(&input);
    }

    MatchResult {
        remainder: input.clone(),
        matched: true,
        matched_values,
    }
}

/// Fulfillment is a result returned by the multiplier_fulfilled function. This is used to determine
/// if a multiplier is fulfilled or not and how.
#[derive(Debug, PartialEq)]
enum Fulfillment {
    /// The multiplier is not yet fulfilled. There must be more values
    NotYetFulfilled,
    /// The multiplier is fulfilled. There cannot be any more values
    Fulfilled,
    /// The multiplied is fulfilled, but there may be more values added
    FulfilledButMoreAllowed,
    /// The multiplier is not fulfilled (ie: too many values).
    NotFulfilled,
}

/// Returns fulfillment enum given the cnt and the actual multiplier of the component
fn multiplier_fulfilled(component: &SyntaxComponent, cnt: usize) -> Fulfillment {
    // Filter out the multipliers that are not relevant for this check
    let binding = component.get_multipliers();
    let mut filtered_multipliers: Vec<_> = binding
        .iter()
        .filter(|m| match m {
            SyntaxComponentMultiplier::AtLeastOneValue => false,
            SyntaxComponentMultiplier::CommaSeparatedRepeat(_, _) => false,
            _ => true,
        })
        .collect();

    // Make sure that whenever we do not find a (primary) multiplier, we use the default "Once".
    if filtered_multipliers.is_empty() {
        filtered_multipliers.push(&SyntaxComponentMultiplier::Once);
    }

    match filtered_multipliers.first().unwrap() {
        SyntaxComponentMultiplier::Once => match cnt {
            0 => Fulfillment::NotYetFulfilled,
            1 => Fulfillment::Fulfilled,
            _ => Fulfillment::NotFulfilled,
        },
        SyntaxComponentMultiplier::ZeroOrMore => Fulfillment::FulfilledButMoreAllowed,
        SyntaxComponentMultiplier::OneOrMore => match cnt {
            0 => Fulfillment::NotYetFulfilled,
            _ => Fulfillment::FulfilledButMoreAllowed,
        },
        SyntaxComponentMultiplier::Optional => match cnt {
            0 => Fulfillment::FulfilledButMoreAllowed,
            1 => Fulfillment::Fulfilled,
            _ => Fulfillment::NotFulfilled,
        },
        SyntaxComponentMultiplier::Between(from, to) => match cnt {
            _ if cnt < *from => Fulfillment::NotYetFulfilled,
            _ if cnt >= *from && cnt <= *to => Fulfillment::FulfilledButMoreAllowed,
            _ => Fulfillment::NotFulfilled,
        },
        _ => Fulfillment::NotFulfilled,
    }
}

/// Helper function to return no matches
fn no_match(input: &Vec<CssValue>) -> MatchResult {
    MatchResult {
        remainder: input.clone(),
        matched: false,
        matched_values: vec![],
    }
}

/// Helper function to return the first element from input in a match result, as we need this a lot
fn first_match(input: &Vec<CssValue>) -> MatchResult {
    MatchResult {
        remainder: input.iter().skip(1).cloned().collect(),
        matched: true,
        matched_values: vec![input.first().unwrap().clone()],
    }
}

#[cfg(test)]
mod tests {
    use gosub_css3::stylesheet::CssValue;

    use crate::css_definitions::{parse_definition_files, PropertyDefinition};
    use crate::syntax::CssSyntax;

    use super::*;

    macro_rules! str {
        ($s:expr) => {
            CssValue::String($s.to_string())
        };
    }

    macro_rules! assert_match {
        ($e:expr) => {
            println!("\n\n-------- ASSERT MATCH --------");
            let res = $e.clone();
            dbg!(&res);
            assert_eq!(true, res.matched);
            println!("------------------------------\n\n");
        };
    }

    macro_rules! assert_not_match {
        ($e:expr) => {
            println!("\n\n------- ASSERT NOT MATCH ------");
            let res = $e;
            dbg!(&res);
            assert_eq!(false, res.matched);
            println!("------------------------------\n\n");
        };
    }

    macro_rules! assert_true {
        ($e:expr) => {
            assert_eq!(true, $e);
        };
    }

    macro_rules! assert_false {
        ($e:expr) => {
            assert_eq!(false, $e);
        };
    }

    #[test]
    fn test_match_group1() {
        // Exactly one
        let tree = CssSyntax::new("auto | none | block").compile().unwrap();
        assert_true!(tree.matches(vec![str!("auto")]));
        assert_true!(tree.matches(vec![CssValue::None]));
        assert_true!(tree.matches(vec![str!("block")]));
        assert_false!(tree.matches(vec![str!("inline")]));
        assert_false!(tree.matches(vec![str!("")]));
        assert_false!(tree.matches(vec![str!("foobar")]));
        assert_false!(tree.matches(vec![str!("foo"), CssValue::None]));
        assert_false!(tree.matches(vec![CssValue::None, str!("foo")]));
        assert_false!(tree.matches(vec![str!("auto"), CssValue::None]));
        assert_false!(tree.matches(vec![str!("auto"), CssValue::Comma, str!("none"),]));
        assert_false!(tree.matches(vec![
            str!("auto"),
            CssValue::Comma,
            CssValue::None,
            CssValue::Comma,
            str!("block"),
        ]));
    }

    #[test]
    fn test_match_group2() {
        // juxtaposition
        let tree = CssSyntax::new("auto none block").compile().unwrap();
        assert_false!(tree.matches(vec![str!("auto")]));
        assert_false!(tree.matches(vec![CssValue::None]));
        assert_false!(tree.matches(vec![str!("block")]));
        assert_true!(tree.matches(vec![str!("auto"), CssValue::None, str!("block"),]));
        assert_false!(tree.matches(vec![str!("block"), CssValue::None, str!("block"),]));
        assert_false!(tree.matches(vec![str!("auto"), CssValue::None, str!("auto"),]));
    }

    #[test]
    fn test_match_group3() {
        // all any order
        let tree = CssSyntax::new("auto && none && block").compile().unwrap();
        assert_false!(tree.matches(vec![str!("auto")]));
        assert_false!(tree.matches(vec![CssValue::None]));
        assert_false!(tree.matches(vec![str!("block")]));
        assert_false!(tree.matches(vec![str!("inline")]));
        assert_false!(tree.matches(vec![str!("")]));
        assert_false!(tree.matches(vec![str!("foobar")]));
        assert_false!(tree.matches(vec![str!("foo"), CssValue::None]));
        assert_false!(tree.matches(vec![CssValue::None, str!("foo")]));
        assert_false!(tree.matches(vec![str!("auto"), CssValue::None]));
        assert_false!(tree.matches(vec![str!("auto"), CssValue::Comma, str!("none")]));
        assert_false!(tree.matches(vec![
            str!("auto"),
            CssValue::Comma,
            CssValue::None,
            CssValue::Comma,
            str!("block")
        ]));
        assert_true!(tree.matches(vec![str!("block"), str!("auto"), CssValue::None]));
        assert_true!(tree.matches(vec![str!("auto"), str!("block"), CssValue::None]));
        assert_true!(tree.matches(vec![str!("block"), CssValue::None, str!("auto")]));
        assert_true!(tree.matches(vec![CssValue::None, str!("auto"), str!("block")]));
        assert_false!(tree.matches(vec![str!("auto"), str!("block")]));
        assert_false!(tree.matches(vec![CssValue::None, str!("block")]));
        assert_false!(tree.matches(vec![
            str!("block"),
            str!("block"),
            CssValue::None,
            CssValue::None
        ]));
    }

    #[test]
    fn test_match_group4() {
        // At least one in any order
        let tree = CssSyntax::new("auto || none || block").compile().unwrap();
        assert_true!(tree.matches(vec![str!("auto")]));
        assert_true!(tree.matches(vec![CssValue::None]));
        assert_true!(tree.matches(vec![str!("block")]));
        assert_true!(tree.matches(vec![str!("auto"), CssValue::None]));
        assert_true!(tree.matches(vec![str!("block"), str!("auto"), CssValue::None,]));

        assert_false!(tree.matches(vec![str!("inline")]));
        assert_false!(tree.matches(vec![str!("")]));
        assert_false!(tree.matches(vec![str!("foo"), CssValue::None]));
        assert_false!(tree.matches(vec![CssValue::None, str!("foo")]));
        assert_false!(tree.matches(vec![CssValue::None, CssValue::None,]));
        assert_false!(tree.matches(vec![str!("auto"), CssValue::Comma, str!("none"),]));
        assert_false!(tree.matches(vec![
            str!("auto"),
            CssValue::Comma,
            CssValue::None,
            CssValue::Comma,
            str!("block"),
        ]));
        assert_false!(tree.matches(vec![
            str!("block"),
            str!("block"),
            CssValue::None,
            CssValue::None,
        ]));
    }

    #[test]
    fn test_match_group_juxtaposition() {
        let tree = CssSyntax::new("auto none block").compile().unwrap();
        if let SyntaxComponent::Group { components, .. } = &tree.components[0] {
            let res = match_group_juxtaposition(&vec![str!("auto")], components);
            assert_not_match!(res);

            let res = match_group_juxtaposition(&vec![str!("auto"), str!("none")], components);
            assert_not_match!(res);

            let res = match_group_juxtaposition(
                &vec![str!("auto"), str!("none"), str!("block")],
                components,
            );
            assert_match!(res);

            let res = match_group_juxtaposition(
                &vec![str!("none"), str!("block"), str!("auto")],
                components,
            );
            assert_not_match!(res);

            let res = match_group_juxtaposition(
                &vec![
                    str!("none"),
                    str!("block"),
                    str!("block"),
                    str!("auto"),
                    str!("none"),
                ],
                components,
            );
            assert_not_match!(res);

            let res = match_group_juxtaposition(
                &vec![str!("none"), str!("banana"), str!("car"), str!("block")],
                components,
            );
            assert_not_match!(res);
        }
    }

    #[test]
    fn test_match_group_juxtaposition_with_groups() {
        // Test if groups are working icw juxtaposition
        let tree = CssSyntax::new("[top | bottom] [ up | down ] [ charm | strange] ")
            .compile()
            .unwrap();
        if let SyntaxComponent::Group { components, .. } = &tree.components[0] {
            let res = match_group_juxtaposition(
                &vec![str!("top"), str!("up"), str!("strange")],
                components,
            );
            assert_match!(res);

            let res = match_group_juxtaposition(
                &vec![str!("bottom"), str!("up"), str!("strange")],
                components,
            );
            assert_match!(res);

            let res = match_group_juxtaposition(
                &vec![str!("bottom"), str!("down"), str!("charm")],
                components,
            );
            assert_match!(res);
        }
    }

    #[test]
    fn test_match_group_all_any_order() {
        let tree = CssSyntax::new("auto none block").compile().unwrap();
        if let SyntaxComponent::Group { components, .. } = &tree.components[0] {
            let res = match_group_all_any_order(&vec![str!("auto")], components);
            assert_not_match!(res);

            let res = match_group_all_any_order(&vec![str!("auto"), str!("none")], components);
            assert_not_match!(res);

            let res = match_group_all_any_order(
                &vec![str!("auto"), str!("none"), str!("block")],
                components,
            );
            assert_match!(res);

            let res = match_group_all_any_order(
                &vec![str!("none"), str!("block"), str!("auto")],
                components,
            );
            assert_match!(res);

            let res = match_group_all_any_order(
                &vec![
                    str!("none"),
                    str!("block"),
                    str!("block"),
                    str!("auto"),
                    str!("none"),
                ],
                components,
            );
            assert_not_match!(res);

            let res = match_group_all_any_order(
                &vec![str!("none"), str!("banana"), str!("car"), str!("block")],
                components,
            );
            assert_not_match!(res);
        }
    }

    #[test]
    fn test_match_group_at_least_one_any_order() {
        let tree = CssSyntax::new("auto none block").compile().unwrap();
        if let SyntaxComponent::Group { components, .. } = &tree.components[0] {
            let res = match_group_at_least_one_any_order(&vec![str!("auto")], components);
            assert_match!(res);

            let res =
                match_group_at_least_one_any_order(&vec![str!("auto"), str!("none")], components);
            assert_match!(res);

            let res = match_group_at_least_one_any_order(
                &vec![str!("auto"), str!("none"), str!("block")],
                components,
            );
            assert_match!(res);

            let res = match_group_at_least_one_any_order(
                &vec![str!("none"), str!("block"), str!("auto")],
                components,
            );
            assert_match!(res);

            let res = match_group_at_least_one_any_order(
                &vec![str!("none"), str!("block"), str!("auto")],
                components,
            );
            assert_match!(res);

            let res = match_group_at_least_one_any_order(
                &vec![
                    str!("none"),
                    str!("block"),
                    str!("none"),
                    str!("block"),
                    str!("auto"),
                    str!("none"),
                ],
                components,
            );
            assert_match!(res);
            assert_eq!(vec![str!("none"), str!("block")], res.matched_values);

            let res = match_group_at_least_one_any_order(
                &vec![str!("none"), str!("block"), str!("banana"), str!("auto")],
                components,
            );
            assert_match!(res);
            assert_eq!(vec![str!("none"), str!("block")], res.matched_values);
            assert_eq!(vec![str!("banana"), str!("auto")], res.remainder);

            let res = match_group_at_least_one_any_order(&vec![], components);
            assert_not_match!(res);
        }
    }

    #[test]
    fn test_multipliers_optional() {
        let tree = CssSyntax::new("foo bar baz").compile().unwrap();
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("baz"),]));
        assert_false!(tree.clone().matches(vec![str!("foo"), str!("baz"),]));

        let tree = CssSyntax::new("foo bar?").compile().unwrap();
        dbg!(&tree);
        assert_true!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree.clone().matches(vec![str!("foo"), str!("bar"),]));
        assert_false!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("bar"),]));
        assert_false!(tree.clone().matches(vec![str!("bar"), str!("foo"),]));

        let tree = CssSyntax::new("foo bar? baz").compile().unwrap();
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree.clone().matches(vec![str!("foo"), str!("baz"),]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("baz"),]));

        assert_false!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
        ]));

        assert_false!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("baz"),
            str!("baz"),
        ]));
    }

    #[test]
    fn test_multipliers_zero_or_more() {
        let tree = CssSyntax::new("foo bar* baz").compile().unwrap();
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("baz"),]));
        assert_true!(tree.clone().matches(vec![str!("foo"), str!("baz"),]));
        assert_true!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
        ]));
        assert_false!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
            str!("bar"),
        ]));

        let tree = CssSyntax::new("foo bar*").compile().unwrap();
        assert_true!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree.clone().matches(vec![str!("foo"), str!("bar"),]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("bar"),]));
        assert_false!(tree.clone().matches(vec![str!("bar"), str!("foo"),]));
    }

    #[test]
    fn test_multipliers_one_or_more() {
        let tree = CssSyntax::new("foo bar+ baz").compile().unwrap();
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("baz"),]));
        assert_false!(tree.clone().matches(vec![str!("foo"), str!("baz"),]));
        assert_true!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
        ]));
        assert_false!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
            str!("bar"),
        ]));

        let tree = CssSyntax::new("foo bar+").compile().unwrap();
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_false!(tree.clone().matches(vec![str!("bar")]));
        assert_true!(tree.clone().matches(vec![str!("foo"), str!("bar"),]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("bar"),]));
        assert_false!(tree.clone().matches(vec![str!("bar"), str!("foo"),]));

        let tree = CssSyntax::new("foo+ bar+").compile().unwrap();
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_false!(tree.clone().matches(vec![str!("bar")]));
        assert_true!(tree.clone().matches(vec![str!("foo"), str!("bar"),]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("bar"),]));
        assert_true!(tree.clone().matches(vec![
            str!("foo"),
            str!("foo"),
            str!("bar"),
            str!("bar"),
        ]));

        assert_false!(tree.clone().matches(vec![str!("bar"), str!("foo"),]));
    }

    #[test]
    fn test_multipliers_between() {
        let tree = CssSyntax::new("foo bar{1,3} baz").compile().unwrap();
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_false!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("baz"),]));
        assert_false!(tree.clone().matches(vec![str!("foo"), str!("baz"),]));
        assert_true!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
        ]));
        assert_true!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
        ]));
        assert_false!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
        ]));
        assert_false!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("baz"),
            str!("bar"),
            str!("bar"),
        ]));

        let tree = CssSyntax::new("foo bar{0,3}").compile().unwrap();
        assert_true!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree.clone().matches(vec![str!("foo")]));
        assert_true!(tree.clone().matches(vec![str!("foo"), str!("bar"),]));
        assert_true!(tree
            .clone()
            .matches(vec![str!("foo"), str!("bar"), str!("bar"),]));
        assert_false!(tree.clone().matches(vec![
            str!("foo"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
            str!("bar"),
        ]));
        assert_false!(tree.clone().matches(vec![str!("bar"), str!("foo"),]));
    }

    #[test]
    fn test_matcher() {
        let mut definitions = parse_definition_files();
        definitions.add_property(
            "testprop",
            PropertyDefinition {
                name: "testprop".to_string(),
                computed: vec![],
                syntax: CssSyntax::new(
                    "[ left | right ] <length>? | [ top | bottom ] <length> | [ top | bottom ]",
                )
                .compile()
                .unwrap(),
                inherited: false,
                initial_value: None,
                resolved: false,
            },
        );
        definitions.resolve();

        let prop = definitions.find_property("testprop").unwrap();

        assert_true!(prop
            .clone()
            .matches(vec![str!("left"), CssValue::Unit(5.0, "px".into()),]));
        assert_true!(prop
            .clone()
            .matches(vec![str!("top"), CssValue::Unit(5.0, "px".into()),]));
        assert_true!(prop
            .clone()
            .matches(vec![str!("bottom"), CssValue::Unit(5.0, "px".into()),]));
        assert_true!(prop
            .clone()
            .matches(vec![str!("right"), CssValue::Unit(5.0, "px".into()),]));
        assert_true!(prop.clone().matches(vec![str!("left")]));
        assert_true!(prop.clone().matches(vec![str!("top")]));
        assert_true!(prop.clone().matches(vec![str!("bottom")]));
        assert_true!(prop.clone().matches(vec![str!("right")]));

        assert_false!(prop
            .clone()
            .matches(vec![CssValue::Unit(5.0, "px".into()), str!("right"),]));
        assert_false!(prop.clone().matches(vec![
            CssValue::Unit(5.0, "px".into()),
            CssValue::Unit(10.0, "px".into()),
            str!("right"),
        ]));
    }

    #[test]
    fn test_matcher_2() {
        let mut definitions = parse_definition_files();
        definitions.add_property(
            "testprop",
            PropertyDefinition {
                name: "testprop".to_string(),
                computed: vec![],
                syntax: CssSyntax::new("[ [ left | center | right | top | bottom | <length-percentage> ] | [ left | center | right | <length-percentage> ] [ top | center | bottom | <length-percentage> ] ]").compile().unwrap(),
                inherited: false,
                initial_value: None,
                resolved: false,
            },
        );
        definitions.resolve();

        let prop = definitions.find_property("testprop").unwrap();

        assert_true!(prop.clone().matches(vec![str!("left"),]));
        assert_true!(prop.clone().matches(vec![str!("left"), str!("top"),]));
        assert_true!(prop.clone().matches(vec![str!("center"), str!("top"),]));
        assert_false!(prop.clone().matches(vec![str!("top"), str!("top"),]));
        assert_false!(prop.clone().matches(vec![str!("top"), str!("center"),]));
        assert_true!(prop.clone().matches(vec![str!("center"), str!("top"),]));
        assert_true!(prop.clone().matches(vec![str!("center"), str!("center"),]));
        assert_true!(prop
            .clone()
            .matches(vec![CssValue::Percentage(10.0), CssValue::Percentage(20.0),]));
        assert_true!(prop.clone().matches(vec![
            CssValue::Unit(10.0, "px".into()),
            CssValue::Percentage(20.0),
        ]));
        assert_true!(prop
            .clone()
            .matches(vec![str!("left"), CssValue::Percentage(20.0),]));

        assert_true!(prop
            .clone()
            .matches(vec![CssValue::Unit(10.0, "px".into()), str!("center"),]));

        assert_true!(prop
            .clone()
            .matches(vec![CssValue::Percentage(10.0), str!("top"),]));

        assert_true!(prop.clone().matches(vec![str!("right")]));

        assert_true!(prop.clone().matches(vec![str!("top")]));
    }

    #[test]
    fn test_matcher_3() {
        let mut definitions = parse_definition_files();
        definitions.add_property(
            "testprop",
            PropertyDefinition {
                name: "testprop".to_string(),
                computed: vec![],
                syntax: CssSyntax::new("foo | [ foo [ foo | bar ] ]")
                    .compile()
                    .unwrap(),
                inherited: false,
                initial_value: None,
                resolved: false,
            },
        );
        definitions.resolve();

        let prop = definitions.find_property("testprop").unwrap();

        assert_true!(prop.clone().matches(vec![str!("foo"),]));
        assert_true!(prop.clone().matches(vec![str!("foo"), str!("foo"),]));
        assert_true!(prop.clone().matches(vec![str!("foo"), str!("bar"),]));

        assert_false!(prop.clone().matches(vec![str!("bar"),]));
        assert_false!(prop.clone().matches(vec![str!("bar"), str!("foo"),]));
    }

    #[test]
    fn test_fulfillment() {
        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::Once],
                },
                0
            ),
            Fulfillment::NotYetFulfilled
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::Once],
                },
                1
            ),
            Fulfillment::Fulfilled
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::Once],
                },
                2
            ),
            Fulfillment::NotFulfilled
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::ZeroOrMore],
                },
                0
            ),
            Fulfillment::FulfilledButMoreAllowed
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::ZeroOrMore],
                },
                1
            ),
            Fulfillment::FulfilledButMoreAllowed
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::ZeroOrMore],
                },
                2
            ),
            Fulfillment::FulfilledButMoreAllowed
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::OneOrMore],
                },
                0
            ),
            Fulfillment::NotYetFulfilled
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::OneOrMore],
                },
                1
            ),
            Fulfillment::FulfilledButMoreAllowed
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::OneOrMore],
                },
                2
            ),
            Fulfillment::FulfilledButMoreAllowed
        );

        assert_eq!(
            multiplier_fulfilled(
                &SyntaxComponent::Group {
                    components: vec![],
                    combinator: GroupCombinators::Juxtaposition,
                    multipliers: vec![SyntaxComponentMultiplier::Optional],
                },
                0
            ),
            Fulfillment::FulfilledButMoreAllowed
        );
    }

    #[test]
    fn test_match_with_subgroups() {
        let tree = CssSyntax::new("[a b ] | [a c]").compile().unwrap();
        assert_true!(tree.matches(vec![str!("a"), str!("b"),]));
        assert_true!(tree.matches(vec![str!("a"), str!("c"),]));
        assert_false!(tree.matches(vec![str!("b"), str!("b"),]));
    }

    #[test]
    fn test_matcher_4() {
        let mut definitions = parse_definition_files();
        definitions.add_property(
            "testprop",
            PropertyDefinition {
                name: "testprop".to_string(),
                computed: vec![],
                syntax: CssSyntax::new(
                    "[ left | right ] <length>? | [ top | bottom ] <length> | [ top | bottom ]", // "left <length>? | top <length> | top"
                )
                .compile()
                .unwrap(),
                inherited: false,
                initial_value: None,
                resolved: false,
            },
        );
        definitions.resolve();

        let prop = definitions.find_property("testprop").unwrap();

        assert_true!(prop
            .clone()
            .matches(vec![str!("left"), CssValue::Unit(10.0, "px".into()),]));
        assert_true!(prop
            .clone()
            .matches(vec![str!("right"), CssValue::Unit(10.0, "px".into()),]));
        assert_true!(prop.clone().matches(vec![str!("left"),]));
        assert_true!(prop.clone().matches(vec![str!("right"),]));

        assert_true!(prop
            .clone()
            .matches(vec![str!("top"), CssValue::Unit(10.0, "px".into()),]));
        assert_true!(prop
            .clone()
            .matches(vec![str!("bottom"), CssValue::Unit(10.0, "px".into()),]));

        assert_true!(prop.clone().matches(vec![str!("top"),]));
        assert_true!(prop.clone().matches(vec![str!("bottom"),]));
    }

    #[test]
    fn test_comma_separated() {
        let tree = CssSyntax::new("[foo | bar | baz]#").compile().unwrap();
        assert_true!(tree.matches(vec![str!("foo")]));
        assert_true!(tree.matches(vec![str!("foo"), CssValue::Comma, str!("foo")]));
        assert_true!(tree.matches(vec![
            str!("foo"),
            CssValue::Comma,
            str!("foo"),
            CssValue::Comma,
            str!("foo")
        ]));
        assert_true!(tree.matches(vec![str!("foo"), CssValue::Comma, str!("bar")]));
        assert_true!(tree.matches(vec![str!("foo"), CssValue::Comma, str!("baz")]));
        assert_true!(tree.matches(vec![
            str!("foo"),
            CssValue::Comma,
            str!("bar"),
            CssValue::Comma,
            str!("baz")
        ]));

        assert_false!(tree.matches(vec![str!("foo"), CssValue::Comma]));
        assert_false!(tree.matches(vec![
            str!("foo"),
            CssValue::Comma,
            str!("bar"),
            CssValue::Comma
        ]));
        assert_false!(tree.matches(vec![
            str!("foo"),
            CssValue::Comma,
            CssValue::Comma,
            str!("bar")
        ]));
    }
}
