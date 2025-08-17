use libp2p::ping;

pub type MessageProtocol = ping::Behaviour;

pub fn new_with_ping(ping_interval: Option<std::time::Duration>) -> ping::Behaviour {
    let mut cfg = ping::Config::default();
    if let Some(int) = ping_interval {
        cfg = cfg.with_interval(int);
    }
    ping::Behaviour::new(cfg)
}