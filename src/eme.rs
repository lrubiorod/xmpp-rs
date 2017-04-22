use minidom::Element;

use error::Error;

use ns;

#[derive(Debug, Clone)]
pub struct ExplicitMessageEncryption {
    pub namespace: String,
    pub name: Option<String>,
}

pub fn parse_explicit_message_encryption(root: &Element) -> Result<ExplicitMessageEncryption, Error> {
    if !root.is("encryption", ns::EME) {
        return Err(Error::ParseError("This is not an encryption element."));
    }
    for _ in root.children() {
        return Err(Error::ParseError("Unknown child in encryption element."));
    }
    let namespace = root.attr("namespace").ok_or(Error::ParseError("Mandatory argument 'namespace' not present in encryption element."))?.to_owned();
    let name = root.attr("name").and_then(|value| value.parse().ok());
    Ok(ExplicitMessageEncryption {
        namespace: namespace,
        name: name,
    })
}

#[cfg(test)]
mod tests {
    use minidom::Element;
    use error::Error;
    use eme;

    #[test]
    fn test_simple() {
        let elem: Element = "<encryption xmlns='urn:xmpp:eme:0' namespace='urn:xmpp:otr:0'/>".parse().unwrap();
        let encryption = eme::parse_explicit_message_encryption(&elem).unwrap();
        assert_eq!(encryption.namespace, "urn:xmpp:otr:0");
        assert_eq!(encryption.name, None);

        let elem: Element = "<encryption xmlns='urn:xmpp:eme:0' namespace='some.unknown.mechanism' name='SuperMechanism'/>".parse().unwrap();
        let encryption = eme::parse_explicit_message_encryption(&elem).unwrap();
        assert_eq!(encryption.namespace, "some.unknown.mechanism");
        assert_eq!(encryption.name, Some(String::from("SuperMechanism")));
    }

    #[test]
    fn test_unknown() {
        let elem: Element = "<replace xmlns='urn:xmpp:message-correct:0'/>".parse().unwrap();
        let error = eme::parse_explicit_message_encryption(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "This is not an encryption element.");
    }

    #[test]
    fn test_invalid_child() {
        let elem: Element = "<encryption xmlns='urn:xmpp:eme:0'><coucou/></encryption>".parse().unwrap();
        let error = eme::parse_explicit_message_encryption(&elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown child in encryption element.");
    }
}