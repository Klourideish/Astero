//! Bounded process-local user session mechanism. No host-account or callback execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    NotInitialized,
    InvalidUser,
    NoEvent,
    Capacity,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Event {
    pub login: bool,
    pub user: i32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Snapshot {
    pub initialized: bool,
    pub logged_in: bool,
    pub user: i32,
    pub pending_events: usize,
    pub generation: u64,
}
pub struct Users {
    snapshot: Snapshot,
    events: [Option<Event>; 2],
}
impl Users {
    pub fn new(user: i32) -> Self {
        Self {
            snapshot: Snapshot {
                initialized: false,
                logged_in: false,
                user,
                pending_events: 0,
                generation: 0,
            },
            events: [None; 2],
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        self.snapshot.clone()
    }
    pub fn initialize(&mut self) {
        if !self.snapshot.initialized {
            self.snapshot.initialized = true;
            self.snapshot.logged_in = true;
            self.snapshot.generation = self.snapshot.generation.saturating_add(1);
            self.events = [
                Some(Event {
                    login: true,
                    user: self.snapshot.user,
                }),
                None,
            ];
            self.snapshot.pending_events = 1;
        }
    }
    pub fn terminate(&mut self) {
        self.snapshot.initialized = false;
        self.snapshot.logged_in = false;
        self.snapshot.pending_events = 0;
        self.events = [None; 2];
    }
    pub fn require_initialized(&self) -> Result<(), Error> {
        if self.snapshot.initialized {
            Ok(())
        } else {
            Err(Error::NotInitialized)
        }
    }
    pub fn validate_user(&self, user: i32) -> Result<(), Error> {
        self.require_initialized()?;
        if user == self.snapshot.user {
            Ok(())
        } else {
            Err(Error::InvalidUser)
        }
    }
    pub fn logout(&mut self, user: i32) -> Result<(), Error> {
        self.validate_user(user)?;
        if self.snapshot.logged_in {
            if self.snapshot.pending_events == 2 {
                return Err(Error::Capacity);
            }
            self.events[self.snapshot.pending_events] = Some(Event { login: false, user });
            self.snapshot.pending_events += 1;
            self.snapshot.logged_in = false;
        }
        Ok(())
    }
    pub fn event(&self) -> Result<Event, Error> {
        self.require_initialized()?;
        self.events[0].ok_or(Error::NoEvent)
    }
    /// Caller commits only after a successful guest copy, under the service lock.
    pub fn consume_event(&mut self) -> Result<(), Error> {
        self.event()?;
        self.events[0] = self.events[1];
        self.events[1] = None;
        self.snapshot.pending_events -= 1;
        Ok(())
    }
}
