#![doc = include_str!("../README.md")]

use druid::{
    widget::{Button, Flex, Label},
    AppLauncher, LocalizedString, PlatformError, Widget, WidgetExt, WindowDesc,
};

mod config;
mod mail;
mod database;

#[tokio::main]
async fn main() -> Result<(), PlatformError> {
    config::init();
    simple_logger::init().expect("Failed to initialize logging");
    let (mail_tx, mail_rx, mail_agent) = mail::MailAgent::new();
    let main_window = WindowDesc::new(ui_builder());
    let data = 0_u32;
    mail_tx
        .send(mail::MainThreadMessages::FetchIMAP)
        .expect("Channel to the mail agent thread has closed");
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
