use crate::css3::node::{Node, NodeType};
use crate::css3::{Css3, Error};

impl Css3<'_> {

    pub fn parse_at_rule_supports_prelude(&mut self) -> Result<Node, Error> {
        log::trace!("parse_at_rule_supports_prelude");

        let loc = self.tokenizer.current_location().clone();

        // @todo: parse supports condition
        let value = self.consume_raw_condition()?;

        Ok(Node::new(NodeType::Raw { value }, loc))
    }
}


#[cfg(test)]
mod tests {
    use simple_logger::SimpleLogger;
    use crate::byte_stream::{ByteStream, Encoding};
    use crate::css3::Css3;
    use crate::css3::walker::Walker;

    #[test]
    fn test_parse_at_rule_supports_prelude() {
        SimpleLogger::new().init().unwrap();

        let mut it = ByteStream::new();
        it.read_from_str("@supports (display: flex)", Some(Encoding::UTF8));
        let mut parser = Css3::new(&mut it);

        let node = parser.parse_at_rule_supports_prelude().unwrap();

        let w = Walker::new(&node);
        assert_eq!(w.walk_to_string(), "@supports (display: flex)")
    }
}
