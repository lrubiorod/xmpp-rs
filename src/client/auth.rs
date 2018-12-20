use std::mem::replace;
use std::str::FromStr;
use futures::{sink, Async, Future, Poll, Stream, future::{ok, err, IntoFuture}};
use minidom::Element;
use sasl::client::mechanisms::{Anonymous, Plain, Scram};
use sasl::client::Mechanism;
use sasl::common::scram::{Sha1, Sha256};
use sasl::common::Credentials;
use tokio_io::{AsyncRead, AsyncWrite};
use try_from::TryFrom;
use xmpp_parsers::sasl::{Auth, Challenge, Failure, Mechanism as XMPPMechanism, Response, Success};

use crate::xmpp_codec::Packet;
use crate::xmpp_stream::XMPPStream;
use crate::{AuthError, Error, ProtocolError};

const NS_XMPP_SASL: &str = "urn:ietf:params:xml:ns:xmpp-sasl";

pub struct ClientAuth<S: AsyncRead + AsyncWrite> {
    future: Box<Future<Item = XMPPStream<S>, Error = Error>>,
}

impl<S: AsyncRead + AsyncWrite + 'static> ClientAuth<S> {
    pub fn new(stream: XMPPStream<S>, creds: Credentials) -> Result<Self, Error> {
        let mechs: Vec<Box<Mechanism>> = vec![
            // TODO: Box::new(|| …
            Box::new(Scram::<Sha256>::from_credentials(creds.clone()).unwrap()),
            Box::new(Scram::<Sha1>::from_credentials(creds.clone()).unwrap()),
            Box::new(Plain::from_credentials(creds).unwrap()),
            Box::new(Anonymous::new()),
        ];

        let mech_names: Vec<String> = stream
            .stream_features
            .get_child("mechanisms", NS_XMPP_SASL)
            .ok_or(AuthError::NoMechanism)?
            .children()
            .filter(|child| child.is("mechanism", NS_XMPP_SASL))
            .map(|mech_el| mech_el.text())
            .collect();
        // TODO: iter instead of collect()
        // println!("SASL mechanisms offered: {:?}", mech_names);

        for mut mechanism in mechs {
            let name = mechanism.name().to_owned();
            if mech_names.iter().any(|name1| *name1 == name) {
                // println!("SASL mechanism selected: {:?}", name);
                let initial = mechanism.initial().map_err(AuthError::Sasl)?;
                let mechanism_name = XMPPMechanism::from_str(&name).map_err(ProtocolError::Parsers)?;

                let send_initial = Box::new(stream.send_stanza(Auth {
                    mechanism: mechanism_name,
                    data: initial,
                }))
                    .map_err(Error::Io);
                let future = Box::new(send_initial.and_then(
                    |stream| Self::handle_challenge(stream, mechanism)
                ).and_then(
                    |stream| stream.restart()
                ));
                return Ok(ClientAuth {
                    future,
                });
            }
        }

        Err(AuthError::NoMechanism)?
    }

    fn handle_challenge(stream: XMPPStream<S>, mut mechanism: Box<Mechanism>) -> Box<Future<Item = XMPPStream<S>, Error = Error>> {
        Box::new(
            stream.into_future()
            .map_err(|(e, _stream)| e.into())
            .and_then(|(stanza, stream)| {
                match stanza {
                    Some(Packet::Stanza(stanza)) => {
                        if let Ok(challenge) = Challenge::try_from(stanza.clone()) {
                            let response = mechanism
                                .response(&challenge.data);
                            Box::new(
                                response
                                    .map_err(|e| AuthError::Sasl(e).into())
                                    .into_future()
                                    .and_then(|response| {
                                        // Send response and loop
                                        stream.send_stanza(Response { data: response })
                                            .map_err(Error::Io)
                                            .and_then(|stream| Self::handle_challenge(stream, mechanism))
                                    })
                            )
                        } else if let Ok(_) = Success::try_from(stanza.clone()) {
                            Box::new(ok(stream))
                        } else if let Ok(failure) = Failure::try_from(stanza.clone()) {
                            Box::new(err(Error::Auth(AuthError::Fail(failure.defined_condition))))
                        } else if stanza.name() == "failure" {
                            // Workaround for https://gitlab.com/xmpp-rs/xmpp-parsers/merge_requests/1
                            Box::new(err(Error::Auth(AuthError::Sasl("failure".to_string()))))
                        } else {
                            // ignore and loop
                            Self::handle_challenge(stream, mechanism)
                        }
                    }
                    Some(_) => {
                        // ignore and loop
                        Self::handle_challenge(stream, mechanism)
                    }
                    None => Box::new(err(Error::Disconnected))
                }
            })
        )
    }
}

impl<S: AsyncRead + AsyncWrite> Future for ClientAuth<S> {
    type Item = XMPPStream<S>;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        self.future.poll()
    }
}
