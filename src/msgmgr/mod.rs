use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Mutex};
use std::thread;
use std::fmt;
use std::any::Any;
use hashbrown::HashMap;
use super::transport;
use super::rmb;

const CONTROLBUS: rmb::Bus = 0;

#[derive(Clone)]
pub struct RmbMsg {
    bus: rmb::Bus,
    msg: Box<dyn rmb::Msg + 'static>,
}


#[derive(Debug,Clone)]
struct SubscribeMsg {
    b: rmb::Bus, 
    f: fn(rmb::Bus, Box<dyn rmb::Msg + 'static>)-> Result<String, String> 
}

impl fmt::Display for SubscribeMsg {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Write strictly the first element into the supplied output
        // stream: `f`. Returns `fmt::Result` which indicates whether the
        // operation succeeded or failed. Note that `write!` uses syntax which
        // is very similar to `println!`.
        write!(f, "SubscribeMsg: {:?}", self.b)
    }
}


pub struct MsgMgr<'a> {
    inited: bool,
    transports: Mutex<Vec<(std::ops::Range<u32>,Box<dyn transport::Transport<'a> + 'a>)>>,
    que: Mutex<queue::Queue<RmbMsg>>,
    subscribers: Mutex<HashMap<rmb::Bus, HashMap<thread::ThreadId, fn(rmb::Bus, Box<dyn rmb::Msg + 'static>)-> Result<String, String> >>>,
    self_tx: Sender<RmbMsg>,
    _thread_tx: Sender<RmbMsg>,
    _self_rx: Receiver<RmbMsg>,
    _thread_rx: Receiver<RmbMsg>,
}

impl<'a> MsgMgr<'a> {
///
/// Returns a Message Manager object
/// 
/// There should only be one MsgMgr per process. (This limitation may change in future versions)
/// 
/// # Arguments
/// A Vector of range/transport pairs for this object to manage.  This allows you to assign different bus ranges
/// to different transport implementors 
/// 
/// # Example
/// ```
/// use msgbus::{msgmgr,transport::{internal,local}};
/// let i = Box::new(internal::TransportInternal::new());
/// let l = Box::new(local::TransportLocal::new());
/// let mm = msgmgr::MsgMgr::new(vec![(1..10,i),(11..20,l)]);
/// ```
/// 
    pub fn new(transports: Vec<(std::ops::Range<rmb::Bus>,Box<dyn transport::Transport<'a> + 'a>)>) -> MsgMgr<'a> {  
        let (st, tr): (Sender<RmbMsg>, Receiver<RmbMsg>) = mpsc::channel();
        let (tt, sr): (Sender<RmbMsg>, Receiver<RmbMsg>) = mpsc::channel();
        let t = Mutex::new(transports);
        MsgMgr { 
            transports: t,
            que: Mutex::new(queue::Queue::new()),
            subscribers: Mutex::new(HashMap::new()),
            inited: false,
            self_tx: st,
            _thread_rx: tr,
            _thread_tx: tt,
            _self_rx: sr,
        } 
    }
///
/// Initialize the Message Manager
///     
    pub fn init(&mut self) -> Result<String, String> {
        let t = self.transports.lock().unwrap();
        if t.is_empty() {
            return Err("MsgMgr has no transports defined".to_string());
        }
        //
        // TODO: Should this registration occur at init time for all buses, or
        // only on demand when you have clients?
        for (bus_range,transport) in t.iter() {
            transport.register(bus_range, MsgMgr::handle_msg).unwrap();
        }
        self.inited = true;
        Ok("Success".to_string()) 
    }
///
/// Checks to see if the Message Manager has been initialized
///     
    pub fn is_inited(&self) -> bool {
        self.inited
    }
///
/// Runs this Message Manager service. 
/// This method is called by the overarching Message Bus when it is ready to run
///     
    pub fn run(incoming: Receiver<RmbMsg>, 
                _outgoing: Sender<RmbMsg>,
                transports: Mutex<Vec<(std::ops::Range<u32>,&'static (dyn transport::Transport + 'static))>>,
                subscribers: Mutex<HashMap<u32, fn(rmb::Bus, Box<dyn rmb::Msg + 'static>)-> Result<String, String> >>) -> Result<String, String> {
        thread::spawn(move|| {
            loop {
                let msg = incoming.recv().unwrap(); // incoming msg from this thread
                if msg.bus == CONTROLBUS {
                    let m = &(*msg.msg).as_any();
                    if let Some(msg) = m.downcast_ref::<SubscribeMsg>() {
                        let subscribers = subscribers.lock().unwrap();
                        let hm = &*subscribers;
                        if hm.contains_key(&msg.b) {
//                            hm[&msg.b][thread::current().id()] = msg;
                        } else {
 //                           hm[&msg.b] = HashMap::new();
 //                           hm[&msg.b][thread::current().id()] = msg;
                        }
                    }
                } else {
                    let transports = transports.lock().unwrap();
                    // publish on each transports which has a matching bus range
                    (*transports).iter()
                                 .filter(|t| t.0.contains(&msg.bus))
                                 .for_each(|t| { 
                                     t.1.publish(msg.bus, &*msg.msg).unwrap(); 
                                     ()
                                });
                }
            }
        } );
        Ok("Success".to_string()) 
    }
///
/// Get the names of transports registered with the message manager
/// 
/// ## Returns
/// Returns a Result object, with the success path containing a vector of Strings
/// with all the registered transport names.
///
    pub fn get_transport_names(&self) -> Result<Vec<String>, String> {
        if !self.inited {
            return Err("Not Inited".to_string());
        }
        let mut v: Vec<String> = Vec::new();
        let tr = self.transports.lock().unwrap();
        for t in tr.iter() {
            v.push(t.1.name().to_string());
        }
        Ok(v)
    }
    pub fn publish(&mut self, bus: rmb::Bus, msg: Box<dyn rmb::Msg + 'a> ) -> Result<String, String> {
        if self.is_inited() {
            if bus == CONTROLBUS {
                return Err("Bus 0 (ControlBus) is for internal use only".to_string());
            }
            let msg = RmbMsg { bus, msg }; 
            self.self_tx.send(msg).unwrap();
            Ok("Success".to_string())
        } else {
            Err("Not Inited".to_string())
        }
    }


    pub fn subscribe(&mut self, bus: rmb::Bus, f: fn(rmb::Bus, Box<dyn rmb::Msg + 'a>)-> Result<String, String>) -> Result<String, String> {
        if self.inited {
            let m = Box::new(SubscribeMsg { b: bus, f: f});
            let sm = RmbMsg { bus: CONTROLBUS, msg: m};
            self.self_tx.send(sm).unwrap();
            Ok("Not Implemented".to_string())
        } else {
            Err("Not Inited".to_string())
        }
    }

    fn handle_msg(&mut self, bus: rmb::Bus, msg: Box<dyn rmb::Msg + 'a>) -> Result<String, String> {
        let mut q = self.que.lock().unwrap();
        let msg = RmbMsg { bus, msg };
        q.queue(msg).unwrap();
        Ok("".to_string())
    }

}
#[cfg(test)]
mod tests {
    use crate::msgmgr;
    use crate::transport::{local,internal};

    #[test]
   fn test_init_success() {
        let t = Box::new(local::TransportLocal::new());
        let mut mm = msgmgr::MsgMgr::new(vec![(0..10,t)]);
        mm.init().unwrap();
    }
    #[test]
   fn test_init_no_transport() {
        let mut t = msgmgr::MsgMgr::new(vec![]);
        let e = t.init();
        assert_eq!(e, Err("MsgMgr has no transports defined".to_string()));
    }
    #[test]
    fn get_transport_names() {
        let it = Box::new(internal::TransportInternal::new());
        let lt = Box::new(local::TransportLocal::new());
        let mut mm = msgmgr::MsgMgr::new(vec![(0..10,it), (11..20, lt)]);
        mm.init().unwrap();
        let names = mm.get_transport_names().unwrap();
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "internal".to_string());
        assert_eq!(names[1], "local".to_string());
    }
}
