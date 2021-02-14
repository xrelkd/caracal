use std::sync::{Arc, Condvar, Mutex};

use crate::{ClipboardWait, Error, Mode};

pub fn new(mode: Mode) -> (Publisher, Subscriber) {
    let inner = Arc::new((Mutex::new(State::Running), Condvar::new()));
    let notifier = Publisher(inner.clone());
    let subscriber = Subscriber { inner, mode };
    (notifier, subscriber)
}

#[derive(Copy, Clone, Debug)]
enum State {
    Running,
    Stopped,
}

#[derive(Debug)]
pub struct Publisher(Arc<(Mutex<State>, Condvar)>);

#[derive(Debug, Clone)]
pub struct Subscriber {
    inner: Arc<(Mutex<State>, Condvar)>,
    mode: Mode,
}

impl Publisher {
    pub fn notify_all(&self) {
        let (lock, condvar) = &*self.0;
        let mut state = lock.lock().unwrap();
        *state = State::Running;
        condvar.notify_all();
    }

    pub fn close(&self) {
        let (lock, condvar) = &*self.0;
        let mut state = lock.lock().unwrap();
        *state = State::Stopped;
        condvar.notify_all();
    }
}

impl Drop for Publisher {
    fn drop(&mut self) { self.close(); }
}

impl ClipboardWait for Subscriber {
    fn wait(&self) -> Result<Mode, Error> {
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        state = condvar.wait(state).unwrap();
        match *state {
            State::Running => Ok(self.mode),
            State::Stopped => Err(Error::NotifierClosed),
        }
    }
}
