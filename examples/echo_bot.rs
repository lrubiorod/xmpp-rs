extern crate futures;
extern crate tokio_core;
extern crate tokio_xmpp;
extern crate jid;
extern crate minidom;

use std::env::args;
use std::process::exit;
use tokio_core::reactor::Core;
use futures::{Future, Stream, Sink, future};
use tokio_xmpp::Client;
use minidom::Element;

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        println!("Usage: {} <jid> <password>", args[0]);
        exit(1);
    }
    let jid = &args[1];
    let password = &args[2];

    // tokio_core context
    let mut core = Core::new().unwrap();
    // Client instance
    let client = Client::new(jid, password, core.handle()).unwrap();

    // Make the two interfaces for sending and receiving independent
    // of each other so we can move one into a closure.
    let (sink, stream) = client.split();
    // Wrap sink in Option so that we can take() it for the send(self)
    // to consume and return it back when ready.
    let mut sink = Some(sink);
    let mut send = move |stanza| {
        sink = Some(
            sink.take().
                expect("sink")
                .send(stanza)
                .wait()
                .expect("sink.send")
        );
    };
    // Main loop, processes events
    let done = stream.for_each(|event| {
        if event.is_online() {
            println!("Online!");

            let presence = make_presence();
            send(presence);
        } else if let Some(stanza) = event.as_stanza() {
            if stanza.name() == "message" &&
                stanza.attr("type") != Some("error") {
                    // This is a message we'll echo
                    let from = stanza.attr("from");
                    let body = stanza.get_child("body", "jabber:client")
                        .map(|el| el.text());

                    match (from, body) {
                        (Some(from), Some(body)) => {
                            let reply = make_reply(from, body);
                            send(reply);
                        },
                        _ => (),
                    };
                }
        }

        Box::new(future::ok(()))
    });

    // Start polling `done`
    match core.run(done) {
        Ok(_) => (),
        Err(e) => {
            println!("Fatal: {}", e);
            ()
        }
    }
}

// Construct a <presence/>
fn make_presence() -> Element {
    Element::builder("presence")
        .append(Element::builder("status")
                .append("chat")
                .build())
        .append(Element::builder("show")
                .append("Echoing messages")
                .build())
        .build()
}

// Construct a chat <message/>
fn make_reply(to: &str, body: String) -> Element {
    Element::builder("message")
        .attr("type", "chat")
        .attr("to", to)
        .append(Element::builder("body")
                .append(body)
                .build())
        .build()
}