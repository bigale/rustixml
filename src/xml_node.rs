//! XML node representation for parse results
//!
//! This module defines the XML output structure produced by the native parser.

/// XML node types for parse results
#[derive(Debug, Clone, PartialEq)]
pub enum XmlNode {
    Element {
        name: String,
        attributes: Vec<(String, String)>,
        children: Vec<XmlNode>,
    },
    Text(String),
    Attribute {
        name: String,
        value: String,
    }, // For @mark - to be extracted by parent
}

impl XmlNode {
    /// Extract text content from a node (for attributes)
    pub fn text_content(&self) -> String {
        match self {
            XmlNode::Text(s) => s.clone(),
            XmlNode::Element { children, .. } => children
                .iter()
                .map(|child| child.text_content())
                .collect::<Vec<_>>()
                .join(""),
            XmlNode::Attribute { value, .. } => value.clone(),
        }
    }

    fn escape_xml_attr(s: &str) -> String {
        // We use single quotes for attribute values
        // Per XML spec, in attributes we must escape: &, <, ' (when using single quotes)
        s.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('\'', "&apos;")
    }

    fn escape_xml_text(s: &str) -> String {
        // In text content, we must escape: &, <
        // Note: > can optionally be escaped but is not required by XML spec
        s.replace('&', "&amp;").replace('<', "&lt;")
    }

    /// Convert to XML string
    pub fn to_xml(&self) -> String {
        self.to_xml_internal(0, "")
    }

    fn to_xml_internal(&self, _depth: usize, _indent: &str) -> String {
        match self {
            XmlNode::Element {
                name,
                attributes,
                children,
            } => {
                let attrs_str = if attributes.is_empty() {
                    String::new()
                } else {
                    format!(
                        " {}",
                        attributes
                            .iter()
                            .map(|(k, v)| format!("{}='{}'", k, Self::escape_xml_attr(v)))
                            .collect::<Vec<_>>()
                            .join(" ")
                    )
                };

                if children.is_empty() {
                    format!("<{}{}/>", name, attrs_str)
                } else {
                    let content: String = children
                        .iter()
                        .map(|child| child.to_xml_internal(_depth + 1, _indent))
                        .collect();
                    format!("<{}{}>{}</{}>", name, attrs_str, content, name)
                }
            }
            XmlNode::Text(s) => Self::escape_xml_text(s),
            XmlNode::Attribute { .. } => {
                // Attributes should have been extracted by parent
                String::new()
            }
        }
    }
}
