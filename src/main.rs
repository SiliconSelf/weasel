#![doc = include_str!("../README.md")]

use actix::Actor;
use druid::{
    widget::{Button, Flex, Label},
    AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc,
};

mod config;
mod database;
mod mail;

use mail::MailActor;

#[actix::main]
async fn main() -> Result<(), PlatformError> {
    config::init();
    simple_logger::init().expect("Failed to initialize logging");

    let config = config::GLOBAL_CONFIG.get().expect("Configuration has not been initialized");

    // Start mail actors for all accounts
    for user in config.get_accounts() {
        MailActor::create(|_| { MailActor::new(user) });
    }

    let main_window = WindowDesc::new(ui_builder());
    let data = 0_u32;
    AppLauncher::with_window(main_window).log_to_console().launch(data)
}

/// Weh
fn ui_builder() -> impl Widget<u32> {
    // The label text will be computed dynamically based on the current locale
    // and count
    let text = LocalizedString::new("hello-counter")
        .with_arg("count", |data: &u32, _env| (*data).into());
    let label = Label::new(text).padding(5.0).center();
    let button = Button::new("increment")
        .on_click(|_ctx, data, _env| *data += 1)
        .padding(5.0);

    Flex::column().with_child(label).with_child(button)
}
