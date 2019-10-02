#![doc(html_playground_url = "https://play.rust-lang.org/")]

//! Work in Progress - not usable yet!
//! 
//! # msgbus
//! `msgbus` provides the ability to publish messages on a channel (or bus) and subscribe to channels to recieve all the messages 
//! published on that channel.  To support scalability, when you first register to be able to publish on that channel, you indicate what 
//! kind of bandwidth you will require.  The overall message bus is managed by a Message Manager(msgmgr) and that msgmgr will be configured 
//! with various transport capabilities.
//! 
//! # Theory of Operation
//! A message bus is a general communication mechanism which has a many-to-many relationship between publishers (many clients can publish on a given bus)
//! and subscribers (many clients can subscribe to a given bus). In an implementaiton where we can have many buses, we can have dedicated buses for one-to-one,
//! one-to-many and many-to-one relationships as needed.
//! 
//! In this particular implementation, we can have many buses, called `Channels`, and a further enhancement has been added to support scalability.  In the
//! simplest implementation, the publishers and subscribers are threads in a shared application.  Communication between them is considered to be high 
//! bandwidth as it can be implemented as shared/copied messages.  In a slightly scaled up implementation, the publishers and subscribers may exist in 
//! separate applicaitons on the some processor. This medium bandwith implementation can be implemented as shared memory between those applications or other
//! local mechanisms.  A lower bandwith implementation may have those puclishers and subscribers existing on different connected processors.
//! 
//! The enhancement to support this is called the `Transport` and is presented as a trait of which we provide several different examples.  The clients 
//! (pubblishers or subscribers) don't choose the transport, but rather the bandwidth they require.  If during later development, the application must be split
//! across mulitple processes, or multiple processors, the clients require almost no refactoring as they are independent from the transport.
//! 
//! # Publishing
//! Publishing is a non block call to the `msgbus` designated by the `rmb::Channel`.  This is a simple `u32` which you define the means for your specific 
//! application. What you send is a structure with the trait of `rmb::Msg`.  The msg will be put on the channel, whether there is any subscribers or not.
//! 
//! # Subscribing
//! When you subscribe to the a particular channel, your handler will be called for all msgs received from that point forward.  The handler may be a function
//! or closure which you passed to the subscribe call.  The handler will be called in the thread context that the msgbus was created in. 
//! 
//! # Simple Example
//! 
//! ```
//! use msgbus::{msgmgr,rmb,transport::internal};
//!
//! fn main() {
//!     let t = internal::TransportInternal::new();
//!     let mut mb = msgmgr::MsgMgr::new(&t);
//!     let mut r = rmb::Rmb::new(&mut mb);
//! 
//! }
pub mod rmb;
pub mod msgmgr;
pub mod transport;

#[cfg(test)]
mod tests {
    use super::{rmb, msgmgr, transport::local, transport::internal};
    #[test]
    fn test_init() {
            let t = local::TransportLocal::new();
        let mut mb = msgmgr::MsgMgr::new(&t);
        let mut r = rmb::Rmb::new(&mut mb);
        r.init().unwrap();
    }
    #[test]
    fn test_simple_subscribe_publish() {
        impl rmb::Msg for String {

        }
        fn handler(_chan: rmb::Channel, msg: &dyn rmb::Msg)-> Result<String, String> {
            println!("{}", msg); 
            assert_eq!(msg.to_string(), "Hello".to_string()); 
            Ok(msg.to_string())
        }

        let t = internal::TransportInternal::new();
        let mut mb = msgmgr::MsgMgr::new(&t);
        let mut r = rmb::Rmb::new(&mut mb);
        r.init().unwrap();
        let hello = "Hello".to_string();
        let chan = 1;
        r.subscribe(chan, handler).unwrap();
        r.publish(chan, &hello).unwrap();
    }
}
