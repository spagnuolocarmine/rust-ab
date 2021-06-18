extern crate priority_queue;

use std::sync::Mutex;

use cfg_if::cfg_if;
use clap::{App, Arg};
use lazy_static::*;
use priority_queue::PriorityQueue;
use rayon::{ThreadPool, ThreadPoolBuilder};

use crate::engine::agent::Agent;
use crate::engine::agentimpl::AgentImpl;
use crate::engine::priority::Priority;
use crate::engine::state::State;

lazy_static! {
    pub static ref THREAD_NUM: usize =
                                {
                                let matches = App::new("Rust-AB").
                                arg(
                                    Arg::with_name("bench").
                                    long("bench")
                                ).
                                    arg(
                                        Arg::with_name("num_thread").
                                        help("sets the number of threads to use")
                                        .takes_value(true).
                                        long("nt")
                                    ).
                                    get_matches();
                                let n = match matches.value_of("num_thread"){
                                    Some(nt) => match nt.parse::<usize>(){
                                                Ok(ris) => ris,
                                                Err(_) => {
                                                    eprintln!("error: --nt value is not an integer");
                                                    num_cpus::get()
                                                }
                                    },
                                    _ => num_cpus::get()
                                };
                                //println!("Using {} threads",n);
                                n
                                };
}
pub struct Schedule {
    pub step: usize,
    pub time: f64,
    pub events: Mutex<PriorityQueue<AgentImpl, Priority>>,
    pub pool: Option<ThreadPool>,
    // Mainly used in the visualization to render newly scheduled agents.
    // This is cleared at the start of each step.
    pub to_visualize: Vec<Box<dyn Agent>>,
}

#[derive(Clone)]
pub struct Pair {
    agentimpl: AgentImpl,
    priority: Priority,
}

impl Pair {
    fn new(agent: AgentImpl, the_priority: Priority) -> Pair {
        Pair {
            agentimpl: agent,
            priority: the_priority,
        }
    }
}

impl Schedule {
    pub fn new() -> Schedule {
        //println!("Using {} thread",*THREAD_NUM);
        cfg_if! {
            if #[cfg(feature ="parallel")]{
                return Schedule {
                    step: 0,
                    time: 0.0,
                    events: Mutex::new(PriorityQueue::new()),
                    pool: Some(ThreadPoolBuilder::new().num_threads(*THREAD_NUM).build().unwrap()),
                    to_visualize: Vec::new()
                }
            }else{
                return Schedule {
                    step: 0,
                    time: 0.0,
                    events: Mutex::new(PriorityQueue::new()),
                    pool: None,
                    to_visualize: Vec::new()
                }
            }
        }
    }

    pub fn with_threads(thread_num: usize) -> Schedule {
        //println!("Using {} thread",thread_num);
        cfg_if! {
            if #[cfg(feature ="parallel")]{
                return Schedule {
                    step: 0,
                    time: 0.0,
                    events: Mutex::new(PriorityQueue::new()),
                    pool: Some(ThreadPoolBuilder::new().num_threads(thread_num).build().unwrap()),
                    to_visualize: Vec::new()
                }
            }else{
                return Schedule {
                    step: 0,
                    time: 0.0,
                    events: Mutex::new(PriorityQueue::new()),
                    pool: None,
                    to_visualize: Vec::new()
                }
            }
        }
    }

    pub fn schedule_once(&mut self, agent: AgentImpl, the_time: f64, the_ordering: i64) {
        if self.step > 0 {
            self.to_visualize.push(agent.agent.clone());
        }
        self.events.lock().unwrap().push(
            agent,
            Priority {
                time: the_time,
                ordering: the_ordering,
            },
        );
    }

    pub fn schedule_repeating(&mut self, agent: Box<dyn Agent>, the_time: f64, the_ordering: i64) {
        let mut a = AgentImpl::new(agent);
        a.repeating = true;
        let pr = Priority::new(the_time, the_ordering);
        self.events.lock().unwrap().push(a, pr);
    }

    pub fn simulate(&mut self, state: &mut Box<&mut dyn State>, num_step: u128) {
        for _ in 0..num_step {
            self.step(state);
        }
    }

    cfg_if! {
        if #[cfg(feature ="parallel")]{


        pub fn step(&mut self,state: &mut Box<&mut dyn State>){
            self.newly_scheduled.clear();
            let thread_num = self.pool.as_ref().unwrap().current_num_threads();

            if self.step == 0{
                state.update(self.step);
            }

            self.step += 1;

            // let start: std::time::Instant = std::time::Instant::now();
            let events = &mut self.events;
            if events.lock().unwrap().is_empty() {
                //println!("coda eventi vuota");
                return
            }

            let thread_division = (events.lock().unwrap().len() as f64 / thread_num as f64).ceil() as usize ;
            let mut cevents: Vec<Vec<Pair>> = vec![Vec::with_capacity(thread_division); thread_num];

            let mut i = 0;

            match events.lock().unwrap().peek() {
                Some(item) => {
                    let (_agent, priority) = item;
                    self.time = priority.time;
                },
                None => panic!("agente non trovato"),
            }

            loop {
                if events.lock().unwrap().is_empty() {
                    break;
                }

                match events.lock().unwrap().peek() {
                    Some(item) => {
                        let (_agent, priority) = item;
                        if priority.time > self.time {
                            break;
                        }
                    },
                    None => panic!("agente non trovato"),
                }

                let item = events.lock().unwrap().pop();
                match item {
                    Some(item) => {
                        let (agent, priority) = item;
                        // let x = agent.id.clone();
                        // println!("{}", x);
                        let index = match thread_num{
                            0 => 0,
                            _ => i%thread_num
                        };
                        cevents[index].push(Pair::new(agent, priority));
                        i+=1;
                    },
                    None => panic!("no item"),
                }
            }

                // state.before_step(...)

            self.pool.as_ref().unwrap().scope( |scope| {
                for _ in 0..thread_num{
                    let batch = cevents.pop().unwrap();
                    scope.spawn(|_| {
                        let mut reschedule = Vec::with_capacity(batch.len());
                        for mut item in batch {
                            item.agentimpl.agent.step(state);
                            let should_remove = item.agentimpl.agent.should_remove(state);
                            let should_reproduce = item.agentimpl.agent.should_reproduce(state);

                            if item.agentimpl.repeating && !should_remove {
                                reschedule.push( ( item.agentimpl, Priority{ time: item.priority.time+1.0, ordering: item.priority.ordering}) );
                            }

                            if let Some(new_agents) = should_reproduce {
                                for (new_agent, schedule_options) in new_agents {
                                    let ScheduleOptions{ordering, repeating} = schedule_options;
                                    //let agent = *new_agent;
                                    let mut new_agent_impl = AgentImpl::new(new_agent.clone());
                                    new_agent_impl.repeating = repeating;
                                    reschedule.push((new_agent_impl, Priority{time: item.priority.time + 1., ordering}));
                                    //self.newly_scheduled.push(new_agent.clone());
                                }
                            }
                        }
                        let mut events = self.events.lock().unwrap();
                        for entry in reschedule{
                            events.push(entry.0,entry.1);
                        }
                    });

                }
            });


        state.update(self.step);
        }
    }
    else{
        pub fn step(&mut self,state: &mut Box<&mut dyn State>){
            state.before_step(self);
            if self.step == 0{
                state.update(self.step);
            }
            self.step += 1;


            // let start: std::time::Instant = std::time::Instant::now();
            let events = &mut self.events;
            if events.lock().unwrap().is_empty() {
                //println!("coda eventi vuota");
                return;
            }

            let mut cevents: Vec<Pair> = Vec::new();

            match events.lock().unwrap().peek() {
                Some(item) => {
                    let (_agent, priority) = item;
                    self.time = priority.time;
                }
                None => panic!("agente non trovato"),
            }

            loop {
                if events.lock().unwrap().is_empty() {
                    break;
                }

                match events.lock().unwrap().peek() {
                    Some(item) => {
                        let (_agent, priority) = item;
                        if priority.time > self.time {
                            break;
                        }
                    }
                    None => panic!("agente non trovato"),
                }

                let item = events.lock().unwrap().pop();
                match item {
                    Some(item) => {
                        let (agent, priority) = item;
                        // let x = agent.id.clone();
                        // println!("{}", x);
                        cevents.push(Pair::new(agent, priority));
                    }
                    None => panic!("no item"),
                }
            }

            for mut item in cevents.into_iter() {

                item.agentimpl.agent.step(state);

                let should_remove = item.agentimpl.agent.should_remove(state);
                //let should_reproduce = item.agentimpl.agent.should_reproduce(state);

                if item.agentimpl.repeating && !should_remove {
                    self.schedule_once(
                        item.agentimpl,
                        item.priority.time + 1.0,
                        item.priority.ordering,
                    );
                }

                /*if let Some(new_agents) = should_reproduce {
                    for (new_agent, schedule_options) in new_agents {
                        let ScheduleOptions{ordering, repeating} = schedule_options;
                        //let agent = *new_agent;
                        let mut new_agent_impl = AgentImpl::new(new_agent.clone());
                        new_agent_impl.repeating = repeating;
                        self.schedule_once(new_agent_impl, item.priority.time + 1., ordering);
                        //self.newly_scheduled.push(new_agent.clone());
                    }
                }*/
            }

            state.update(self.step);
            // println!("Time spent calling step method, step {} : {:?}",self.step,start.elapsed());
            state.after_step(self);

            self.to_visualize.clear();

            }

        }
    }
}

/// A struct used to specify schedule options to pass to an agent's clone when an agent reproduces.
pub struct ScheduleOptions {
    pub ordering: i64,
    pub repeating: bool,
}
