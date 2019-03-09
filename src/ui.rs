use std::error::Error;

use gtk::prelude::*;

use super::ipc::{Connection, Output};

pub fn run(_connection: Connection, outputs: Vec<Output>) -> Result<(), Box<Error>> {
    gtk::init()?;
    let dialog = gtk::Dialog::new_with_buttons(
        Some("Display Settings"),
        gtk::NONE_WINDOW,
        gtk::DialogFlags::empty(),
        &[],
    );
    let dialog_box = dialog.get_content_area();
    for output in &outputs {
        let frame = gtk::Frame::new(&*format!("Output {}", output.name));
        dialog_box.add(&frame);

        let frame_box = gtk::Box::new(gtk::Orientation::Horizontal, 10);
        frame.add(&frame_box);

        let enabled = gtk::CheckButton::new_with_label("Turn On");
        enabled.set_active(output.active);
        if outputs.len() == 1 {
            enabled.set_sensitive(false);
        }
        frame_box.add(&enabled);

        let resolution = gtk::ComboBoxText::new();
        let refresh = gtk::ComboBoxText::new();
        for mode in &output.modes {
            let res_text = format!("{}x{}", mode.width, mode.height);
            resolution.append(None, &res_text);
            let refresh_text = format!("{}.{:03}", mode.refresh / 1000, mode.refresh % 1000);
            refresh.append(None, &refresh_text);
        }
        frame_box.add(&resolution);
        frame_box.add(&refresh);
    }
    dialog.show_all();
    dialog.run();
    Ok(())
}
