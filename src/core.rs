use std::rc::Rc;
use std::cell::RefCell;
use std::collections::HashMap;

use state::State;
use mio::{Token, EventLoop, Handler, EventSet};

// TODO make this FnOnce
pub type CoreMessage = Box<FnMut(&mut Core, &mut EventLoop<Core>) + Send>;
pub type CoreTimeout = ();

#[derive(Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Debug)]
pub struct Context(usize);

pub struct Core {
    token_counter: usize,
    context_counter: usize,
    contexts: HashMap<Token, Context>,
    states: HashMap<Context, Rc<RefCell<State>>>,
}

impl Core {
    pub fn new() -> Self {
        Core {
            token_counter: 0,
            context_counter: 0,
            contexts: HashMap::new(),
            states: HashMap::new(),
        }
    }

    pub fn get_new_token(&mut self) -> Token {
        let token_counter = self.token_counter;
        self.token_counter = token_counter.wrapping_add(1);

        Token(token_counter)
    }

    pub fn get_token(&mut self) -> Token {
        Token(self.token_counter.clone())
    }

    pub fn get_new_context(&mut self) -> Context {
        let context_counter = self.context_counter;
        self.context_counter = context_counter.wrapping_add(1);

        Context(context_counter)
    }

    pub fn insert_context(&mut self, key: Token, val: Context) -> Option<Context> {
        self.contexts.insert(key, val)
    }

    pub fn insert_state(&mut self,
                        key: Context,
                        val: Rc<RefCell<State>>)
                        -> Option<Rc<RefCell<State>>> {
        self.states.insert(key, val)
    }

    pub fn remove_context(&mut self, key: &Token) -> Option<Context> {
        self.contexts.remove(key)
    }

    pub fn remove_state(&mut self, key: &Context) -> Option<Rc<RefCell<State>>> {
        self.states.remove(key)
    }

    pub fn get_context(&self, key: &Token) -> Option<&Context> {
        self.contexts.get(key)
    }

    pub fn get_state(&self, key: &Context) -> Option<&Rc<RefCell<State>>> {
        self.states.get(key)
    }
}

impl Handler for Core {
    type Timeout = CoreTimeout;
    type Message = CoreMessage;

    fn ready(&mut self, event_loop: &mut EventLoop<Self>, token: Token, events: EventSet) {
        let state = match self.contexts.get(&token) {
            Some(context) => {
                match self.states.get(context) {
                    Some(state) => state.clone(),
                    None => return,
                }
            }
            None => return,
        };

        state.borrow_mut().execute(self, event_loop, token, events);
    }

    fn notify(&mut self, event_loop: &mut EventLoop<Self>, mut msg: Self::Message) {
        msg(self, event_loop);
    }
}
