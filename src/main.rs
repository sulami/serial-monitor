use std::io::prelude::*;
use std::io::{self, stdin, stdout, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

use serial_core::{SerialDevice, SerialPortSettings};
use serial_unix::{TTYPort, TTYSettings};

fn main() -> io::Result<()> {
    let mut tty = TTYPort::open(Path::new("/dev/tty.usbmodemTEST1"))?;
    tty.set_timeout(std::time::Duration::new(5, 0))?;
    let mut settings: TTYSettings = tty.read_settings()?;
    settings.set_baud_rate(serial_core::Baud9600)?;
    tty.write_settings(&settings)?;

    let tty_arc = Arc::new(Mutex::new(tty));

    let output_arc = Arc::clone(&tty_arc);
    thread::spawn(move || {
        output_handler(output_arc);
    });

    let input_arc = Arc::clone(&tty_arc);
    let input = thread::spawn(move || {
        input_handler(input_arc);
    });

    input.join().unwrap();

    Ok(())
}

fn input_handler(tty_mutex: Arc<Mutex<TTYPort>>) {
    loop {
        let mut buf = String::new();
        stdin().read_line(&mut buf).unwrap();
        match buf.as_str() {
            "/quit\n" => {
                println!("Bye!");
                return;
            }
            _ => {
                let mut tty = tty_mutex.lock().unwrap();
                write!(tty, "{}", buf).unwrap();
            }
        }
    }
}

fn output_handler(tty_mutex: Arc<Mutex<TTYPort>>) -> ! {
    let mut char_buf = [0u8];
    loop {
        let mut tty = tty_mutex.lock().unwrap();
        tty.read_exact(&mut char_buf).unwrap();
        print!("{}", String::from_utf8(char_buf.to_vec()).unwrap());
        stdout().flush().unwrap();
    }
}
