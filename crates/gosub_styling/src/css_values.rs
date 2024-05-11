use core::fmt::Debug;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;

use gosub_css3::stylesheet::{
    CssOrigin, CssSelector, CssSelectorPart, CssSelectorType, MatcherType, Specificity,
};
use gosub_html5::node::NodeId;
use gosub_html5::parser::document::DocumentHandle;

use crate::css_colors::RgbColor;

// Matches a complete selector (all parts) against the given node(id)
pub(crate) fn match_selector(
    document: DocumentHandle,
    node_id: NodeId,
    selector: &CssSelector,
) -> bool {
    let mut parts = selector.parts.clone();
    parts.reverse();
    match_selector_part(document, node_id, &mut parts)
}

/// Returns true when the given node matches the part(s)
fn match_selector_part(
    document: DocumentHandle,
    node_id: NodeId,
    selector_parts: &mut Vec<CssSelectorPart>,
) -> bool {
    let binding = document.get();
    let mut next_current_node = Some(binding.get_node_by_id(node_id).expect("node not found"));

    while !selector_parts.is_empty() {
        if next_current_node.is_none() {
            return false;
        }
        let current_node = next_current_node.expect("current_node not found");
        if current_node.is_root() {
            return false;
        }

        let part = selector_parts.remove(0);

        match part.type_ {
            CssSelectorType::Universal => {
                // '*' always matches any selector
            }
            CssSelectorType::Type => {
                if !current_node.is_element() {
                    return false;
                }
                if part.value != current_node.as_element().name {
                    return false;
                }
            }
            CssSelectorType::Class => {
                if !current_node.is_element() {
                    return false;
                }
                if !current_node.as_element().classes.contains(&part.value) {
                    return false;
                }
            }
            CssSelectorType::Id => {
                if !current_node.is_element() {
                    return false;
                }
                if current_node
                    .as_element()
                    .attributes
                    .get("id")
                    .unwrap_or(&String::new())
                    != &part.value
                {
                    return false;
                }
            }
            CssSelectorType::Attribute => {
                let wanted_attr_name = part.name;

                if !current_node.has_attribute(&wanted_attr_name) {
                    return false;
                }

                let mut wanted_attr_value = part.value;
                let mut got_attr_value = current_node
                    .get_attribute(&wanted_attr_name)
                    .unwrap_or(&String::new())
                    .to_string();

                // If we need to match case-insensitive, just convert everything to lowercase for comparison
                if part.flags.eq_ignore_ascii_case("i") {
                    wanted_attr_value = wanted_attr_value.to_lowercase();
                    got_attr_value = got_attr_value.to_lowercase();
                };

                return match part.matcher {
                    MatcherType::None => {
                        // Just the presence of the attribute is enough
                        true
                    }
                    MatcherType::Equals => {
                        // Exact match
                        wanted_attr_value == got_attr_value
                    }
                    MatcherType::Includes => {
                        // Contains word
                        wanted_attr_value
                            .split_whitespace()
                            .any(|s| s == got_attr_value)
                    }
                    MatcherType::DashMatch => {
                        // Exact value or value followed by a hyphen
                        got_attr_value == wanted_attr_value
                            || got_attr_value.starts_with(&format!("{wanted_attr_value}-"))
                    }
                    MatcherType::PrefixMatch => {
                        // Starts with
                        got_attr_value.starts_with(&wanted_attr_value)
                    }
                    MatcherType::SuffixMatch => {
                        // Ends with
                        got_attr_value.ends_with(&wanted_attr_value)
                    }
                    MatcherType::SubstringMatch => {
                        // Contains
                        got_attr_value.contains(&wanted_attr_value)
                    }
                };
            }
            CssSelectorType::PseudoClass => {
                // @Todo: implement pseudo classes
                if part.value == "link" {
                    return false;
                }
                return false;
            }
            CssSelectorType::PseudoElement => {
                // @Todo: implement pseudo elements
                if part.value == "first-child" {
                    return false;
                }
                return false;
            }
            CssSelectorType::Combinator => {
                // We don't have the descendant combinator (space), as this is the default behaviour
                match part.value.as_str() {
                    // @todo: We also should do: column combinator ('||' experimental)
                    // @todo: Namespace combinator ('|')
                    " " => {
                        // Descendant combinator, any parent that matches the previous selector will do
                        if !match_selector_part(
                            DocumentHandle::clone(&document),
                            current_node.id,
                            selector_parts,
                        ) {
                            // we insert the combinator back so we the next loop will match against the parent node
                            selector_parts.insert(0, part);
                        }
                    }
                    ">" => {
                        // Child combinator. Only matches the direct child
                        if !match_selector_part(
                            DocumentHandle::clone(&document),
                            current_node.id,
                            selector_parts,
                        ) {
                            return false;
                        }
                    }
                    "+" => {
                        // We need to match the previous sibling of the current node
                    }
                    "~" => {
                        // We need to match the previous siblings of the current node
                    }
                    _ => {
                        panic!("Unknown combinator: {}", part.value);
                    }
                }
            }
        }

        // We have matched this part, so we move up the chain
        // let binding = document.get();
        next_current_node = binding.parent_node(current_node);
    }

    // All parts of the selector have matched
    true
}

/// A declarationProperty defines a single value for a property (color: red;). It consists of the value,
/// origin, importance, location and specificity of the declaration.
#[derive(Debug, Clone)]
pub struct DeclarationProperty {
    /// The actual value of the property
    pub value: CssValue,
    /// Origin of the declaration (user stylesheet, author stylesheet etc.)
    pub origin: CssOrigin,
    /// Whether the declaration is !important
    pub important: bool,
    /// The location of the declaration in the stylesheet (name.css:123) or empty
    pub location: String,
    /// The specificity of the selector that declared this property
    pub specificity: Specificity,
}

impl DeclarationProperty {
    /// Priority of the declaration based on the origin and importance as defined in https://developer.mozilla.org/en-US/docs/Web/CSS/Cascade
    fn priority(&self) -> u8 {
        match self.origin {
            CssOrigin::UserAgent => {
                if self.important {
                    7
                } else {
                    1
                }
            }
            CssOrigin::User => {
                if self.important {
                    6
                } else {
                    2
                }
            }
            CssOrigin::Author => {
                if self.important {
                    5
                } else {
                    3
                }
            }
        }
    }
}

impl PartialEq<Self> for DeclarationProperty {
    fn eq(&self, other: &Self) -> bool {
        self.priority() == other.priority()
    }
}

impl PartialOrd<Self> for DeclarationProperty {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for DeclarationProperty {}

impl Ord for DeclarationProperty {
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority().cmp(&other.priority())
    }
}

/// A value entry contains all values for a single property for a single node. It contains the declared values, and
/// all the computed values.
#[derive(Debug, Clone)]
pub struct CssProperty {
    /// The name of the property
    pub name: String,
    /// True when this property needs to be recalculated
    pub dirty: bool,
    /// List of all declared values for this property
    pub declared: Vec<DeclarationProperty>,
    /// Cascaded value from the declared values (if any)
    pub cascaded: Option<CssValue>,
    // Specified value from the cascaded value (if any), or inherited value, or initial value
    pub specified: CssValue,
    // Computed value from the specified value (needs viewport size etc.)
    pub computed: CssValue,
    pub used: CssValue,
    // Actual value used in the rendering (after rounding, clipping etc.)
    pub actual: CssValue,
}

impl CssProperty {
    #[must_use]
    pub fn new(prop_name: &str) -> Self {
        Self {
            name: prop_name.to_string(),
            dirty: true,
            declared: Vec::new(),
            cascaded: None,
            specified: CssValue::None,
            computed: CssValue::None,
            used: CssValue::None,
            actual: CssValue::None,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Returns the actual value of the property. Will compute the value when needed
    pub fn compute_value(&mut self) -> &CssValue {
        if self.dirty {
            self.calculate_value();
            self.dirty = false;
        }

        &self.actual
    }

    fn calculate_value(&mut self) {
        self.cascaded = self.find_cascaded_value();
        self.specified = self.find_specified_value();
        self.computed = self.find_computed_value();
        self.used = self.find_used_value();
        self.actual = self.find_actual_value();
    }

    fn find_cascaded_value(&self) -> Option<CssValue> {
        let mut declared = self.declared.clone();

        declared.sort();
        declared.sort_by(|a, b| {
            if a.priority() == b.priority() {
                return Ordering::Equal;
            }

            a.specificity.cmp(&b.specificity)
        });

        declared.last().map(|d| d.value.clone())
    }

    fn find_specified_value(&self) -> CssValue {
        match self.declared.iter().max() {
            Some(decl) => decl.value.clone(),
            None => CssValue::None,
        }
    }

    fn find_computed_value(&self) -> CssValue {
        if self.specified != CssValue::None {
            return self.specified.clone();
        }

        if self.is_inheritable() {
            todo!("inheritable properties")
            // while let Some(parent) = self.get_parent() {
            //     if let Some(parent_value) = parent {
            //         return parent_value.find_computed_value();
            //     }
            // }
        }

        self.get_initial_value().unwrap_or(CssValue::None)
    }

    fn find_used_value(&self) -> CssValue {
        self.computed.clone()
    }

    fn find_actual_value(&self) -> CssValue {
        // @TODO: stuff like clipping and such should occur as well
        match &self.used {
            CssValue::Number(len) => CssValue::Number(len.round()),
            CssValue::Percentage(perc) => CssValue::Percentage(perc.round()),
            CssValue::Unit(value, unit) => CssValue::Unit(value.round(), unit.clone()),
            _ => self.used.clone(),
        }
    }

    // Returns true when the property is inheritable, false otherwise
    fn is_inheritable(&self) -> bool {
        crate::property_list::PROPERTY_TABLE
            .iter()
            .find(|entry| entry.name == self.name)
            .is_some_and(|entry| entry.inheritable)
    }

    // Returns the initial value for the property, if any
    fn get_initial_value(&self) -> Option<CssValue> {
        crate::property_list::PROPERTY_TABLE
            .iter()
            .find(|entry| entry.name == self.name)
            .map(|entry| entry.initial.clone())
    }
}

/// Map of all declared values for a single node. Note that these are only the defined properties, not
/// the non-existing properties.
#[derive(Debug)]
pub struct CssProperties {
    pub properties: HashMap<String, CssProperty>,
}

impl Default for CssProperties {
    fn default() -> Self {
        Self::new()
    }
}

impl CssProperties {
    #[must_use]
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }

    pub fn get(&mut self, name: &str) -> Option<&mut CssProperty> {
        self.properties.get_mut(name)
    }
}

/// Actual CSS value, can be a color, length, percentage, string or unit. Some relative values will be computed
/// from other values (ie: Percent(50) will convert to Length(100) when the parent width is 200)
#[derive(Debug, Clone, PartialEq)]
pub enum CssValue {
    None,
    Color(RgbColor),
    Number(f32),
    Percentage(f32),
    String(String),
    Unit(f32, String),
}

impl Display for CssValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Color(col) => {
                write!(
                    f,
                    "#{:02x}{:02x}{:02x}{:02x}",
                    col.r as u8, col.g as u8, col.b as u8, col.a as u8
                )
            }
            Self::Number(num) => write!(f, "{num}"),
            Self::Percentage(p) => write!(f, "{p}%"),
            Self::String(s) => write!(f, "{s}"),
            Self::Unit(val, unit) => write!(f, "{val}{unit}"),
        }
    }
}

impl CssValue {
    #[must_use]
    pub fn to_color(&self) -> Option<RgbColor> {
        match self {
            Self::Color(col) => Some(*col),
            Self::String(s) => Some(RgbColor::from(s.as_str())),
            _ => None,
        }
    }

    #[must_use]
    pub fn unit_to_px(&self) -> f32 {
        //TODO: Implement the rest of the units
        match self {
            Self::Unit(val, unit) => match unit.as_str() {
                "px" => *val,
                "em" => *val * 16.0,
                "rem" => *val * 16.0,
                _ => *val,
            },
            Self::String(value) => {
                if value.ends_with("px") {
                    value.trim_end_matches("px").parse::<f32>().unwrap_or(0.0)
                } else if value.ends_with("rem") {
                    value.trim_end_matches("rem").parse::<f32>().unwrap_or(0.0) * 16.0
                } else if value.ends_with("em") {
                    value.trim_end_matches("em").parse::<f32>().unwrap_or(0.0) * 16.0
                } else {
                    0.0
                }
            }
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compare_declared() {
        let a = DeclarationProperty {
            value: CssValue::String("red".into()),
            origin: CssOrigin::Author,
            important: false,
            location: String::new(),
            specificity: Specificity::new(1, 0, 0),
        };
        let b = DeclarationProperty {
            value: CssValue::String("blue".into()),
            origin: CssOrigin::UserAgent,
            important: false,
            location: String::new(),
            specificity: Specificity::new(1, 0, 0),
        };
        let c = DeclarationProperty {
            value: CssValue::String("green".into()),
            origin: CssOrigin::User,
            important: false,
            location: String::new(),
            specificity: Specificity::new(1, 0, 0),
        };
        let d = DeclarationProperty {
            value: CssValue::String("yellow".into()),
            origin: CssOrigin::Author,
            important: true,
            location: String::new(),
            specificity: Specificity::new(1, 0, 0),
        };
        let e = DeclarationProperty {
            value: CssValue::String("orange".into()),
            origin: CssOrigin::UserAgent,
            important: true,
            location: String::new(),
            specificity: Specificity::new(1, 0, 0),
        };
        let f = DeclarationProperty {
            value: CssValue::String("purple".into()),
            origin: CssOrigin::User,
            important: true,
            location: String::new(),
            specificity: Specificity::new(1, 0, 0),
        };

        assert_eq!(3, a.priority());
        assert_eq!(1, b.priority());
        assert_eq!(2, c.priority());
        assert_eq!(5, d.priority());
        assert_eq!(7, e.priority());
        assert_eq!(6, f.priority());

        assert!(a > b);
        assert!(b < c);
        assert!(c < d);
        assert!(d < e);
        assert!(f < e);
        assert!(a < e);
        assert!(b < d);
        assert!(a < d);
        assert!(b < d);
        assert!(c < d);
        assert_eq!(c, c);
        assert_eq!(d, d);
    }
}
