use super::parser::document::{Document, DocumentHandle};
use crate::html5::node::data::comment::CommentData;
use crate::html5::node::data::document::DocumentData;
use crate::html5::node::data::element::ElementData;
use crate::html5::node::data::text::TextData;
use core::fmt::Debug;
use derive_more::Display;
use std::collections::HashMap;

pub const HTML_NAMESPACE: &str = "http://www.w3.org/1999/xhtml";
pub const MATHML_NAMESPACE: &str = "http://www.w3.org/1998/Math/MathML";
pub const SVG_NAMESPACE: &str = "http://www.w3.org/2000/svg";
pub const XLINK_NAMESPACE: &str = "http://www.w3.org/1999/xlink";
pub const XML_NAMESPACE: &str = "http://www.w3.org/XML/1998/namespace";
pub const XMLNS_NAMESPACE: &str = "http://www.w3.org/2000/xmlns/";

pub mod arena;
pub mod data;

/// Different types of nodes
#[derive(Debug, PartialEq)]
pub enum NodeType {
    Document,
    Text,
    Comment,
    Element,
}

/// Different type of node data
#[derive(Debug, Clone, PartialEq)]
pub enum NodeData {
    /// Represents a document
    Document(DocumentData),
    /// Represents a text
    Text(TextData),
    /// Represents a comment
    Comment(CommentData),
    /// Represents an element
    Element(Box<ElementData>),
}

/// Id used to identify a node
#[derive(Copy, Debug, Default, Eq, Hash, PartialEq, Display, PartialOrd)]
pub struct NodeId(pub(crate) usize);

impl From<NodeId> for usize {
    /// Converts a NodeId into a usize
    fn from(value: NodeId) -> Self {
        value.0
    }
}

impl From<usize> for NodeId {
    /// Converts a usize into a NodeId
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl Default for &NodeId {
    /// Returns the default NodeId, which is 0
    fn default() -> Self {
        &NodeId(0)
    }
}

impl NodeId {
    // TODO: Drop Default derive and only use 0 for the root, or choose another id for the root
    pub const ROOT_NODE: usize = 0;

    /// Returns the root node ID
    pub fn root() -> Self {
        Self(Self::ROOT_NODE)
    }

    /// Returns true when this nodeId is the root node
    pub fn is_root(&self) -> bool {
        self.0 == Self::ROOT_NODE
    }

    /// Returns the next node ID
    pub fn next(&self) -> Self {
        if self.0 == usize::MAX {
            return Self(usize::MAX);
        }

        Self(self.0 + 1)
    }

    /// Returns the nodeID as usize
    pub fn as_usize(&self) -> usize {
        self.0
    }

    /// Returns the previous node ID
    pub fn prev(&self) -> Self {
        if self.0 == 0 {
            return Self::root();
        }

        Self(self.0 - 1)
    }
}

/// Node structure that resembles a DOM node
#[derive(PartialEq)]
pub struct Node {
    /// ID of the node, 0 is always the root / document node
    pub id: NodeId,
    /// Named ID of the node, from the "id" attribute on an HTML element
    pub named_id: Option<String>,
    /// parent of the node, if any
    pub parent: Option<NodeId>,
    /// children of the node
    pub children: Vec<NodeId>,
    /// name of the node, or empty when it's not a tag
    pub name: String,
    /// namespace of the node
    pub namespace: Option<String>,
    /// actual data of the node
    pub data: NodeData,
    /// pointer to document this node is attached to
    pub document: DocumentHandle,

    // Returns true when the given node is registered into an arena
    pub is_registered: bool,
}

impl Node {
    /// Returns true when the given node is of the given namespace
    pub(crate) fn is_namespace(&self, namespace: &str) -> bool {
        self.namespace == Some(namespace.into())
    }

    /// Returns true if the given node is a html integration point
    /// See: https://html.spec.whatwg.org/multipage/parsing.html#html-integration-point
    pub(crate) fn is_html_integration_point(&self) -> bool {
        let namespace = self.namespace.clone().unwrap_or_default();

        if namespace == MATHML_NAMESPACE && self.name == "annotation-xml" {
            if let NodeData::Element(element) = &self.data {
                if let Some(value) = element.attributes.get("encoding") {
                    if value.eq_ignore_ascii_case("text/html") {
                        return true;
                    }
                    if value.eq_ignore_ascii_case("application/xhtml+xml") {
                        return true;
                    }
                }
            }
        }

        namespace == SVG_NAMESPACE
            && ["foreignObject", "desc", "title"].contains(&self.name.as_str())
    }

    /// Returns true if the given node is a mathml integration point
    /// See: https://html.spec.whatwg.org/multipage/parsing.html#mathml-text-integration-point
    pub(crate) fn is_mathml_integration_point(&self) -> bool {
        let namespace = self.namespace.clone().unwrap_or_default();

        namespace == MATHML_NAMESPACE
            && ["mi", "mo", "mn", "ms", "mtext"].contains(&self.name.as_str())
    }
}

impl Debug for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut debug = f.debug_struct("Node");
        debug.field("id", &self.id);
        debug.field("named_id", &self.named_id);
        debug.field("parent", &self.parent);
        debug.field("children", &self.children);
        debug.field("name", &self.name);
        match &self.namespace {
            Some(namespace) if namespace == HTML_NAMESPACE => debug.field("namespace", &"HTML"),
            Some(namespace) if namespace == XML_NAMESPACE => debug.field("namespace", &"XML"),
            Some(namespace) if namespace == XMLNS_NAMESPACE => debug.field("namespace", &"XMLNS"),
            Some(namespace) if namespace == MATHML_NAMESPACE => debug.field("namespace", &"MATHML"),
            Some(namespace) if namespace == SVG_NAMESPACE => debug.field("namespace", &"SVG"),
            Some(namespace) if namespace == XLINK_NAMESPACE => debug.field("namespace", &"XLINK"),
            None => debug.field("namespace", &"None"),
            _ => debug.field("namespace", &"unknown"),
        };
        debug.field("data", &self.data);
        debug.finish()
    }
}

impl Node {
    /// This will only compare against the tag, namespace and data same except element data.
    /// for element data compaare against the tag, namespace and attributes without order.
    /// Both nodes could still have other parents and children.
    pub fn matches_tag_and_attrs_without_order(&self, other: &Self) -> bool {
        if self.name != other.name || self.namespace != other.namespace {
            return false;
        }

        if self.type_of() != other.type_of() {
            return false;
        }

        match self.type_of() {
            NodeType::Element => {
                let mut self_attributes = None;
                let mut other_attributes = None;
                if let NodeData::Element(element) = &self.data {
                    self_attributes = Some(element.attributes.clone_map());
                }
                if let NodeData::Element(element) = &other.data {
                    other_attributes = Some(element.attributes.clone_map());
                }
                self_attributes.eq(&other_attributes)
            }
            _ => self.data == other.data
        }
    }
}

impl Clone for Node {
    fn clone(&self) -> Self {
        Node {
            id: self.id,
            named_id: self.named_id.clone(),
            parent: self.parent,
            children: self.children.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            data: self.data.clone(),
            document: Document::clone(&self.document),
            is_registered: self.is_registered,
        }
    }
}

impl Node {
    /// Create a new document node
    pub fn new_document(document: &DocumentHandle) -> Self {
        Node {
            id: Default::default(),
            named_id: None,
            parent: None,
            children: vec![],
            data: NodeData::Document(DocumentData::new()),
            name: "".to_string(),
            namespace: None,
            document: Document::clone(document),
            is_registered: false,
        }
    }

    /// Create a new element node with the given name and attributes and namespace
    pub fn new_element(
        document: &DocumentHandle,
        name: &str,
        attributes: HashMap<String, String>,
        namespace: &str,
    ) -> Self {
        Node {
            id: Default::default(),
            named_id: None,
            parent: None,
            children: vec![],
            data: NodeData::Element(Box::new(ElementData::with_name_and_attributes(
                Default::default(),
                Document::clone(document),
                name,
                attributes,
            ))),
            name: name.to_string(),
            namespace: Some(namespace.into()),
            document: Document::clone(document),
            is_registered: false,
        }
    }

    /// Creates a new comment node
    pub fn new_comment(document: &DocumentHandle, value: &str) -> Self {
        Node {
            id: Default::default(),
            named_id: None,
            parent: None,
            children: vec![],
            data: NodeData::Comment(CommentData::with_value(value)),
            name: "".to_string(),
            namespace: None,
            document: Document::clone(document),
            is_registered: false,
        }
    }

    /// Creates a new text node
    pub fn new_text(document: &DocumentHandle, value: &str) -> Self {
        Node {
            id: Default::default(),
            named_id: None,
            parent: None,
            children: vec![],
            data: NodeData::Text(TextData::with_value(value)),
            name: "".to_string(),
            namespace: None,
            document: Document::clone(document),
            is_registered: false,
        }
    }

    /// Returns true if the given node is a "formatting" node
    pub fn is_formatting(&self) -> bool {
        self.namespace == Some(HTML_NAMESPACE.into())
            && FORMATTING_HTML_ELEMENTS.contains(&self.name.as_str())
    }

    /// Returns true if the given node is "special" node based on the namespace and name
    pub fn is_special(&self) -> bool {
        if self.namespace == Some(HTML_NAMESPACE.into())
            && SPECIAL_HTML_ELEMENTS.contains(&self.name.as_str())
        {
            return true;
        }
        if self.namespace == Some(MATHML_NAMESPACE.into())
            && SPECIAL_MATHML_ELEMENTS.contains(&self.name.as_str())
        {
            return true;
        }
        if self.namespace == Some(SVG_NAMESPACE.into())
            && SPECIAL_SVG_ELEMENTS.contains(&self.name.as_str())
        {
            return true;
        }

        false
    }

    /// Checks if node has a named ID
    pub fn has_named_id(&self) -> bool {
        if self.type_of() != NodeType::Element {
            return false;
        }

        self.named_id.is_some()
    }

    /// Sets named ID (only applies to Element type, does nothing otherwise)
    pub fn set_named_id(&mut self, named_id: &str) {
        if self.type_of() == NodeType::Element {
            self.named_id = Some(named_id.to_owned());
            if let NodeData::Element(element) = &mut self.data {
                element.attributes.insert("id", named_id);
            }
        }
    }

    /// Gets named ID. If not present or type is not Element, returns None
    pub fn get_named_id(&self) -> Option<String> {
        if self.type_of() != NodeType::Element {
            return None;
        }

        if !self.has_named_id() {
            return None;
        }

        // don't want to return the actual internal String
        self.named_id.clone()
    }

    /// Returns true if this node is registered into an arena
    pub fn is_registered(&self) -> bool {
        self.is_registered
    }
}

pub trait NodeTrait {
    /// Returns the token type of the given token
    fn type_of(&self) -> NodeType;
}

// Each node implements the NodeTrait and has a type_of that will return the node type.
impl NodeTrait for Node {
    /// Returns the token type of the given token
    fn type_of(&self) -> NodeType {
        match self.data {
            NodeData::Document { .. } => NodeType::Document,
            NodeData::Text { .. } => NodeType::Text,
            NodeData::Comment { .. } => NodeType::Comment,
            NodeData::Element { .. } => NodeType::Element,
        }
    }
}

/// HTML elements that are considered formatting elements
pub static FORMATTING_HTML_ELEMENTS: [&str; 14] = [
    "a", "b", "big", "code", "em", "font", "i", "nobr", "s", "small", "strike", "strong", "tt", "u",
];

/// HTML elements that are considered special elements
pub static SPECIAL_HTML_ELEMENTS: [&str; 83] = [
    "address",
    "applet",
    "area",
    "article",
    "aside",
    "base",
    "basefont",
    "bgsound",
    "blockquote",
    "body",
    "br",
    "button",
    "caption",
    "center",
    "col",
    "colgroup",
    "dd",
    "details",
    "dir",
    "div",
    "dl",
    "dt",
    "embed",
    "fieldset",
    "figcaption",
    "figure",
    "footer",
    "form",
    "frame",
    "frameset",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "head",
    "header",
    "hgroup",
    "hr",
    "html",
    "iframe",
    "img",
    "input",
    "keygen",
    "li",
    "link",
    "listing",
    "main",
    "marquee",
    "menu",
    "meta",
    "nav",
    "noembed",
    "noframes",
    "noscript",
    "object",
    "ol",
    "p",
    "param",
    "plaintext",
    "pre",
    "script",
    "search",
    "section",
    "select",
    "source",
    "style",
    "summary",
    "table",
    "tbody",
    "td",
    "template",
    "textarea",
    "tfoot",
    "th",
    "thead",
    "title",
    "tr",
    "track",
    "ul",
    "wbr",
    "xmp",
];

/// MathML elements that are considered special elements
pub static SPECIAL_MATHML_ELEMENTS: [&str; 6] = ["mi", "mo", "mn", "ms", "mtext", "annotation-xml"];

/// SVG elements that are considered special elements
pub static SPECIAL_SVG_ELEMENTS: [&str; 3] = ["foreignObject", "desc", "title"];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::html5::parser::document::Document;

    #[test]
    fn new_document() {
        let document = Document::shared();
        let node = Node::new_document(&document);
        assert_eq!(node.id, NodeId::default());
        assert_eq!(node.parent, None);
        assert!(node.children.is_empty());
        assert_eq!(node.name, "".to_string());
        assert_eq!(node.namespace, None);
        match &node.data {
            NodeData::Document(_) => (),
            _ => panic!(),
        }
    }

    #[test]
    fn new_element() {
        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        let document = Document::shared();
        let node = Node::new_element(&document, "div", attributes.clone(), HTML_NAMESPACE);
        assert_eq!(node.id, NodeId::default());
        assert_eq!(node.parent, None);
        assert!(node.children.is_empty());
        assert_eq!(node.name, "div".to_string());
        assert_eq!(node.namespace, Some(HTML_NAMESPACE.into()));
        let NodeData::Element(element) = &node.data else {
            panic!()
        };
        assert_eq!(element.name, "div");
        assert!(element.attributes.contains("id"));
        assert_eq!(element.attributes.get("id").unwrap(), "test");
    }

    #[test]
    fn new_comment() {
        let document = Document::shared();
        let node = Node::new_comment(&document, "test");
        assert_eq!(node.id, NodeId::default());
        assert_eq!(node.parent, None);
        assert!(node.children.is_empty());
        assert_eq!(node.name, "".to_string());
        assert_eq!(node.namespace, None);
        let NodeData::Comment(CommentData { value, .. }) = &node.data else {
            panic!()
        };
        assert_eq!(value, "test");
    }

    #[test]
    fn new_text() {
        let document = Document::shared();
        let node = Node::new_text(&document, "test");
        assert_eq!(node.id, NodeId::default());
        assert_eq!(node.parent, None);
        assert!(node.children.is_empty());
        assert_eq!(node.name, "".to_string());
        assert_eq!(node.namespace, None);
        let NodeData::Text(TextData { value }) = &node.data else {
            panic!()
        };
        assert_eq!(value, "test");
    }

    #[test]
    fn is_special() {
        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        let document = Document::shared();
        let node = Node::new_element(&document, "div", attributes, HTML_NAMESPACE);
        assert!(node.is_special());
    }

    #[test]
    fn type_of() {
        let document = Document::shared();
        let node = Node::new_document(&document);
        assert_eq!(node.type_of(), NodeType::Document);
        let node = Node::new_text(&document, "test");
        assert_eq!(node.type_of(), NodeType::Text);
        let node = Node::new_comment(&document, "test");
        assert_eq!(node.type_of(), NodeType::Comment);
        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        let node = Node::new_element(&document, "div", attributes, HTML_NAMESPACE);
        assert_eq!(node.type_of(), NodeType::Element);
    }

    #[test]
    fn special_html_elements() {
        let document = Document::shared();
        for element in SPECIAL_HTML_ELEMENTS.iter() {
            let mut attributes = HashMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            let node = Node::new_element(&document, element, attributes, HTML_NAMESPACE);
            assert!(node.is_special());
        }
    }

    #[test]
    fn special_mathml_elements() {
        let document = Document::shared();
        for element in SPECIAL_MATHML_ELEMENTS.iter() {
            let mut attributes = HashMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            let node = Node::new_element(&document, element, attributes, MATHML_NAMESPACE);
            assert!(node.is_special());
        }
    }

    #[test]
    fn special_svg_elements() {
        let document = Document::shared();
        for element in SPECIAL_SVG_ELEMENTS.iter() {
            let mut attributes = HashMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            let node = Node::new_element(&document, element, attributes, SVG_NAMESPACE);
            assert!(node.is_special());
        }
    }

    #[test]
    fn type_of_node() {
        let document = Document::shared();
        let node = Node::new_document(&document);
        assert_eq!(node.type_of(), NodeType::Document);
        let node = Node::new_text(&document, "test");
        assert_eq!(node.type_of(), NodeType::Text);
        let node = Node::new_comment(&document, "test");
        assert_eq!(node.type_of(), NodeType::Comment);
        let mut attributes = HashMap::new();
        attributes.insert("id".to_string(), "test".to_string());
        let node = Node::new_element(&document, "div", attributes, HTML_NAMESPACE);
        assert_eq!(node.type_of(), NodeType::Element);
    }

    #[test]
    fn contains_attribute() {
        let mut attr = HashMap::new();
        attr.insert("x".to_string(), "value".to_string());
        let document = Document::shared();
        let node = Node::new_element(&document, "node", attr.clone(), HTML_NAMESPACE);
        let NodeData::Element(element) = &node.data else {
            panic!()
        };
        assert!(element.attributes.contains("x"));
        assert!(!element.attributes.contains("z"));
    }

    #[test]
    fn insert_attribute() {
        let attr = HashMap::new();
        let document = Document::shared();
        let mut node = Node::new_element(&document, "name", attr.clone(), HTML_NAMESPACE);
        let NodeData::Element(element) = &mut node.data else {
            panic!()
        };
        element.attributes.insert("key", "value");
        assert_eq!(element.attributes.get("key").unwrap(), "value");
    }

    #[test]
    fn remove_attribute() {
        let mut attr = HashMap::new();
        attr.insert("key".to_string(), "value".to_string());
        let document = Document::shared();
        let mut node = Node::new_element(&document, "name", attr.clone(), HTML_NAMESPACE);
        let NodeData::Element(element) = &mut node.data else {
            panic!()
        };
        element.attributes.remove("key");
        assert!(!element.attributes.contains("key"));
    }

    #[test]
    fn get_attribute() {
        let mut attr = HashMap::new();
        attr.insert("key".to_string(), "value".to_string());
        let document = Document::shared();
        let node = Node::new_element(&document, "name", attr.clone(), HTML_NAMESPACE);
        let NodeData::Element(element) = &node.data else {
            panic!()
        };
        assert_eq!(element.attributes.get("key").unwrap(), "value");
    }

    #[test]
    fn get_mut_attribute() {
        let mut attr = HashMap::new();
        attr.insert("key".to_string(), "value".to_string());
        let document = Document::shared();
        let mut node = Node::new_element(&document, "name", attr.clone(), HTML_NAMESPACE);
        let NodeData::Element(element) = &mut node.data else {
            panic!()
        };
        let attr_val = element.attributes.get_mut("key").unwrap();
        attr_val.push_str(" appended");
        assert_eq!(element.attributes.get("key").unwrap(), "value appended");
    }

    #[test]
    fn clear_attributes() {
        let mut attr = HashMap::new();
        attr.insert("key".to_string(), "value".to_string());
        let document = Document::shared();
        let mut node = Node::new_element(&document, "name", attr.clone(), HTML_NAMESPACE);
        let NodeData::Element(element) = &mut node.data else {
            panic!()
        };
        element.attributes.clear();
        assert!(element.attributes.is_empty());
    }

    #[test]
    fn has_attributes() {
        let attr = HashMap::new();
        let document = Document::shared();
        let mut node = Node::new_element(&document, "name", attr.clone(), HTML_NAMESPACE);
        let NodeData::Element(element) = &mut node.data else {
            panic!()
        };
        assert!(element.attributes.is_empty());
        element.attributes.insert("key", "value");
        assert!(!element.attributes.is_empty());
    }
}
