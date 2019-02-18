use std::error::Error;

mod ipc;
mod ui;

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => {
            eprintln!("{}", err);
        }
    }
}

fn run() -> Result<(), Box<Error>> {
    let mut connection = ipc::Connection::connect()?;
    let outputs = connection.get_outputs()?;
    ui::run(connection, outputs)?;
    Ok(())
}
