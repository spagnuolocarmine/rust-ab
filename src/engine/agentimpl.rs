use crate::engine::agent::Agent;
use dyn_clone::DynClone;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

static mut COUNTER: u32 = 0;
#[derive(Clone)]
pub struct AgentImpl {
    pub id: u32,
    pub agent: Box<dyn Agent>,
    pub repeating: bool,
}

impl AgentImpl {
    pub fn new(the_agent: Box<dyn Agent>) -> AgentImpl {
        unsafe {
            COUNTER += 1;

            AgentImpl {
                id: COUNTER,
                agent: the_agent,
                repeating: false,
            }
        }
    }

    pub fn id(self) -> u32 {
        self.id
    }
}

impl fmt::Display for AgentImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.id, self.repeating)
    }
}

impl PartialEq for AgentImpl {
    fn eq(&self, other: &AgentImpl) -> bool {
        self.id == other.id
    }
}

impl Hash for AgentImpl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl Eq for AgentImpl {}
