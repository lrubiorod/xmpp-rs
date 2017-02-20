extern crate xmpp;

use xmpp::jid::Jid;
use xmpp::client::ClientBuilder;
use xmpp::plugins::messaging::{MessagingPlugin, MessageEvent};
use xmpp::plugins::presence::{PresencePlugin, Show};

use std::env;

fn main() {
    let jid: Jid = env::var("JID").unwrap().parse().unwrap();
    let mut client = ClientBuilder::new(jid).connect().unwrap();
    client.register_plugin(MessagingPlugin::new());
    client.register_plugin(PresencePlugin::new());
    client.connect_plain(&env::var("PASS").unwrap()).unwrap();
    client.plugin::<PresencePlugin>().set_presence(Show::Available, None).unwrap();
    loop {
        let event = client.next_event().unwrap();
        if let Some(evt) = event.downcast::<MessageEvent>() {
            println!("{:?}", evt);
        }
    }
}
