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

    'outer: loop {
        if let Ok(mut tty) = TTYPort::open(&args.path) {
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
            loop {
                print!("> ");
                stdout().flush().unwrap();
                let mut buf = String::new();
                stdin().read_line(&mut buf).unwrap();
                match buf.as_str() {
                    "/quit\n" => {
                        println!("Bye!");
                        return Ok(());
                    }
                    _ => {
                        if let Ok(mut tty) = input_arc.lock() {
                            let write_result = write!(tty, "{}", buf);
                            drop(tty);
                            if write_result.is_err() {
                                continue 'outer;
                            }
                            // Wait a bit here to avoid looping back up and
                            // writing our prompt into output that's still being
                            // printed. Because we dropped the TTY, the mutex is
                            // released and the output handler can do its thing.
                            thread::sleep(Duration::from_millis(200));
                        }
                    }
                }
            }
        } else {
            // Failed to open TTY.
            println!("Device offline, waiting...");
            thread::sleep(Duration::from_secs(5));
            continue;
        }
    }
}

fn output_handler(tty_mutex: Arc<Mutex<TTYPort>>) {
    loop {
        thread::sleep(Duration::from_micros(100));
        let mut char_buf = [0u8; 255];
        let mut tty = tty_mutex.lock().unwrap();
        if tty.read(&mut char_buf).is_ok() {
            if let Ok(string) = String::from_utf8(char_buf.to_vec()) {
                print!("{}", string);
                stdout().flush().unwrap();
            } else {
                for c in char_buf {
                    print!("\\x{:02X?}", c);
                }
                stdout().flush().unwrap();
            }
        }
    }
}
