use std::io::prelude::*;
use std::io::{self, stdout, Write};
use std::path::Path;

use serial_core::{SerialDevice, SerialPortSettings};
use serial_unix::{TTYPort, TTYSettings};

fn main() -> io::Result<()> {
    let mut tty = TTYPort::open(Path::new("/dev/tty.usbmodemTEST1"))?;
    tty.set_timeout(std::time::Duration::new(5, 0))?;
    let mut settings: TTYSettings = tty.read_settings()?;
    settings.set_baud_rate(serial_core::Baud9600)?;
    tty.write_settings(&settings)?;
    let mut char_buf = [0u8];
    loop {
        tty.read_exact(&mut char_buf)?;
        print!("{}", String::from_utf8(char_buf.to_vec()).unwrap());
        stdout().flush()?;
    }

    // write!(tty, "Hey there.")?;
}
