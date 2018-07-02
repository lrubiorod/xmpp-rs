// Copyright (c) 2017 Emmanuel Gil Peyrot <linkmauve@linkmauve.fr>
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use helpers::Base64;
use iq::IqSetPayload;

generate_id!(StreamId);

generate_attribute!(Stanza, "stanza", {
    Iq => "iq",
    Message => "message",
}, Default = Iq);

generate_element!(Open, "open", IBB,
attributes: [
    block_size: u16 = "block-size" => required,
    sid: StreamId = "sid" => required,
    stanza: Stanza = "stanza" => default,
]);

impl IqSetPayload for Open {}

generate_element_with_text!(Data, "data", IBB,
    [
        seq: u16 = "seq" => required,
        sid: StreamId = "sid" => required
    ],
    data: Base64<Vec<u8>>
);

impl IqSetPayload for Data {}

generate_element!(Close, "close", IBB,
attributes: [
    sid: StreamId = "sid" => required,
]);

impl IqSetPayload for Close {}

#[cfg(test)]
mod tests {
    use super::*;
    use try_from::TryFrom;
    use minidom::Element;
    use error::Error;
    use std::error::Error as StdError;

    #[test]
    fn test_simple() {
        let sid = StreamId(String::from("coucou"));

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='3' sid='coucou'/>".parse().unwrap();
        let open = Open::try_from(elem).unwrap();
        assert_eq!(open.block_size, 3);
        assert_eq!(open.sid, sid);
        assert_eq!(open.stanza, Stanza::Iq);

        let elem: Element = "<data xmlns='http://jabber.org/protocol/ibb' seq='0' sid='coucou'>AAAA</data>".parse().unwrap();
        let data = Data::try_from(elem).unwrap();
        assert_eq!(data.seq, 0);
        assert_eq!(data.sid, sid);
        assert_eq!(data.data, vec!(0, 0, 0));

        let elem: Element = "<close xmlns='http://jabber.org/protocol/ibb' sid='coucou'/>".parse().unwrap();
        let close = Close::try_from(elem).unwrap();
        assert_eq!(close.sid, sid);
    }

    #[test]
    fn test_invalid() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'block-size' missing.");

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='-5'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseIntError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message.description(), "invalid digit found in string");

        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(error) => error,
            _ => panic!(),
        };
        assert_eq!(message, "Required attribute 'sid' missing.");
    }

    #[test]
    fn test_invalid_stanza() {
        let elem: Element = "<open xmlns='http://jabber.org/protocol/ibb' block-size='128' sid='coucou' stanza='fdsq'/>".parse().unwrap();
        let error = Open::try_from(elem).unwrap_err();
        let message = match error {
            Error::ParseError(string) => string,
            _ => panic!(),
        };
        assert_eq!(message, "Unknown value for 'stanza' attribute.");
    }
}
