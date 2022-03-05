use std::io::prelude::*;
use std::io::{self, stdin, stdout, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use clap::Parser;
use serial_core::BaudRate::{self, *};
use serial_core::SerialPort;
use serial_unix::TTYPort;

fn parse_baud_rate(s: &str) -> Result<BaudRate, &'static str> {
    match s {
        "110" => Ok(Baud110),
        "300" => Ok(Baud300),
        "600" => Ok(Baud600),
        "1200" => Ok(Baud1200),
        "2400" => Ok(Baud2400),
        "4800" => Ok(Baud4800),
        "9600" => Ok(Baud9600),
        "19200" => Ok(Baud19200),
        "38400" => Ok(Baud38400),
        "57600" => Ok(Baud57600),
        "115200" => Ok(Baud115200),
        _ => Err("Unsupported baud rate"),
    }
}

#[derive(Parser)]
#[clap(author, version, about)]
struct Args {
    /// Path of the TTY device, e.g. /dev/tty123
    #[clap(required = true, short, long, parse(from_os_str))]
    path: PathBuf,

    /// Baud rate
    #[clap(short, long, parse(try_from_str = parse_baud_rate), default_value = "9600")]
    baud: serial_core::BaudRate,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let mut tty = TTYPort::open(&args.path)?;
    let _ = tty.reconfigure(&|settings| {
        settings.set_baud_rate(args.baud).unwrap();
        Ok(())
    });

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
        print!("> ");
        stdout().flush().unwrap();
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
                drop(tty);
                // Wait a bit here to avoid looping back up and
                // writing our prompt into output that's still being
                // printed. Because we dropped the TTY, the mutex is
                // released and the output handler can do its thing.
                thread::sleep(Duration::from_millis(10));
            }
        }
    }
}

fn output_handler(tty_mutex: Arc<Mutex<TTYPort>>) -> ! {
    let mut char_buf = [0u8];
    loop {
        // At 9600 Baud we get a new byte every 105 micros. Sleep a
        // while to give other threads a chance to capture the TTY.
        thread::sleep(Duration::from_micros(100));
        let mut tty = tty_mutex.lock().unwrap();
        if tty.read_exact(&mut char_buf).is_ok() {
            print!("{}", String::from_utf8(char_buf.to_vec()).unwrap());
            stdout().flush().unwrap();
        }
    }
}
