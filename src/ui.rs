use std::error::Error;

use gtk::prelude::*;
use gtk::{Inhibit, Window, WindowType};

use super::ipc::{Connection, Response};

pub fn run(connection: Connection, response: Response) -> Result<(), Box<Error>> {
    gtk::init()?;
    let window = Window::new(WindowType::Toplevel);
    window.show_all();
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    gtk::main();
    Ok(())
}