//! Contains the actor that stops the program when the GUI is closed

use actix::prelude::*;

/// An actor to receive a message when the GUI is stopped
pub(crate) struct GuiWatchdogActor;

impl Actor for GuiWatchdogActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::trace!("GUI Watchdog Actor started");
    }
}

impl GuiWatchdogActor {
    /// Create a new watchdog actor
    pub(crate) fn new() -> Self {
        Self
    }
}

/// A message to send to the watchdog actor when the GUI loop has been stopped
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub(crate) struct StopMessage;

impl Handler<StopMessage> for GuiWatchdogActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: StopMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        log::trace!("GUI Watchdog Actor received {msg:?}");
        let system = System::current();
        system.stop();
    }
}
