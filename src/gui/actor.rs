//! Contains the actor for GUI operations

use actix::prelude::*;
use druid::{
    widget::{Button, Flex, Label},
    AppLauncher, LocalizedString, Widget, WidgetExt, WindowDesc,
};

use super::watchdog_actor::GuiWatchdogActor;
use crate::gui::watchdog_actor;

/// An actor that handles rendering the GUI with druid
pub(crate) struct GuiActor {
    /// TODO: Delete this
    watchdog_addr: Addr<GuiWatchdogActor>,
}

impl Actor for GuiActor {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        log::trace!("GUI Actor started");
    }
}

impl GuiActor {
    /// Create a new GUI actor
    pub(crate) fn new(watchdog_addr: Addr<GuiWatchdogActor>) -> Self {
        Self {
            watchdog_addr,
        }
    }
}

/// Start rendering the GUI
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub(crate) struct StartMessage;

impl Handler<StartMessage> for GuiActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: StartMessage,
        _ctx: &mut Context<Self>,
    ) -> Self::Result {
        log::trace!("GuiActor received {msg:?}");
        let main_window = WindowDesc::new(ui_builder());
        let data = 0_u32;
        AppLauncher::with_window(main_window)
            .log_to_console()
            .launch(data)
            .expect("GUI Panicked");
        log::trace!("Window has closed.");
        self.watchdog_addr.do_send(watchdog_actor::StopMessage);
    }
}

/// Build UI
fn ui_builder() -> impl Widget<u32> {
    // The label text will be computed dynamically based on the current
    // locale and count
    let text = LocalizedString::new("hello-counter")
        .with_arg("count", |data: &u32, _env| (*data).into());
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("increment")
        .on_click(move |_ctx, data, _env| {
            *data += 1;
        })
        .padding(5.0);
    Flex::column().with_child(label).with_child(button)
}
