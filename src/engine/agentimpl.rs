use crate::engine::agent::Agent;
use std::clone::Clone;
use std::fmt;
use std::hash::Hash;
use std::hash::Hasher;

static mut COUNTER: u32 = 0;
#[derive(Clone, Debug)]
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

impl<A: Agent + Clone> fmt::Display for AgentImpl {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.id, self.repeating)
    }
}

impl<A: Agent + Clone> PartialEq for AgentImpl {
    fn eq(&self, other: &AgentImpl) -> bool {
        self.id == other.id
    }
}

impl<A: Agent + Clone> Hash for AgentImpl {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl<A: Agent + Clone> Eq for AgentImpl {}
