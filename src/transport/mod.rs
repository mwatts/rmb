use super::rmb;
pub mod internal;
pub mod local;
pub mod network;

#[derive(Debug,PartialEq)]
pub enum Bandwidth {
    Low,
    Medium,
    High,
}

pub trait Transport {
    fn name(&self) -> &'static str;
    fn bandwidth(&self) -> &Bandwidth;
    fn register(&self, channels: &std::ops::Range<rmb::Channel>, handler: fn(rmb::Channel, &dyn rmb::Msg)-> Result<String, String>) -> Result<String, String>;
    fn publish(&self, ch: rmb::Channel, msg: &dyn rmb::Msg) -> Result<String, String>;
}

#[cfg(test)]
mod tests {
    use crate::transport::{Transport, Bandwidth, local, internal, network};

    #[test]
    fn test_init() {
        let t = local::TransportLocal::new();
        t.init().unwrap();
    }
    #[test]
    fn get_local_name() {
        let t = local::TransportLocal::new();
        assert_eq!(t.name(), "local");
        assert_eq!(*t.bandwidth(), Bandwidth::Medium)
    }
    #[test]
    fn get_internal_name() {
        let t = internal::TransportInternal::new();
        assert_eq!(t.name(), "internal");
        assert_eq!(*t.bandwidth(), Bandwidth::High)
    }
    #[test]
    fn get_network_name() {
        let t = network::TransportNetwork::new();
        assert_eq!(t.name(), "network");
        assert_eq!(*t.bandwidth(), Bandwidth::Low)

    }
}