// main.rs
use clap::{Parser, Subcommand};
use serialport::SerialPort;

use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::time::{Duration, Instant};

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    port: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Probe,

    Read {
        output: String,
    },

    Write {
        input: String,
    },
}

fn open_port(name: &str) -> Box<dyn SerialPort> {
    serialport::new(name, 921600)
        .timeout(Duration::from_secs(10))
        .open()
        .expect("Failed to open serial port")
}

fn probe_flash_size(port: &mut Box<dyn SerialPort>) -> usize {
    port.write_all(b"PROBE\n").unwrap();

    let mut response = Vec::new();
    let mut buf = [0u8; 1];

    loop {
        port.read_exact(&mut buf).unwrap();

        if buf[0] == b'\n' {
            break;
        }

        response.push(buf[0]);
    }

    let text = String::from_utf8(response).unwrap();

    text.trim()
        .parse::<usize>()
        .expect("Device returned invalid flash size")
}

fn main() {
    let cli = Cli::parse();

    let mut port = open_port(&cli.port);

    match cli.command {
        Commands::Probe => {
            let size = probe_flash_size(&mut port);

            println!(
                "Flash size: {} bytes ({:.2} MB)",
                size,
                size as f64 / 1024.0 / 1024.0
            );
        }

        Commands::Read { output } => {
            let flash_size = probe_flash_size(&mut port);

            println!(
                "Reading {:.2} MB...",
                flash_size as f64 / 1024.0 / 1024.0
            );

            port.write_all(b"READ\n").unwrap();

            let file =
                File::create(&output).unwrap();

            let mut file =
                BufWriter::with_capacity(
                    1024 * 1024,
                    file,
                );

            let mut buffer =
                vec![0u8; 65536];

            let mut received: usize = 0;

            let start = Instant::now();

            while received < flash_size {
                let remaining =
                    flash_size - received;

                let wanted =
                    remaining.min(buffer.len());

                let n = port
                    .read(&mut buffer[..wanted])
                    .unwrap();

                if n == 0 {
                    panic!("Unexpected EOF");
                }

                file.write_all(&buffer[..n])
                    .unwrap();

                received += n;

                let percent =
                    (received as f64
                        / flash_size as f64)
                        * 100.0;

                print!(
                    "\r{:6.2}% ({}/{})",
                    percent,
                    received,
                    flash_size
                );

                std::io::stdout()
                    .flush()
                    .unwrap();
            }

            file.flush().unwrap();

            let elapsed =
                start.elapsed().as_secs_f64();

            let mbps =
                (received as f64
                    / 1024.0
                    / 1024.0)
                    / elapsed;

            println!();
            println!(
                "Completed in {:.2}s",
                elapsed
            );

            println!(
                "Speed: {:.2} MB/s",
                mbps
            );

            println!(
                "Saved to {}",
                output
            );
        }

        Commands::Write { input } => {
            let data =
                std::fs::read(&input).unwrap();

            println!(
                "Writing {} bytes...",
                data.len()
            );

            let cmd =
                format!("WRITE:{}\n", data.len());

            port.write_all(cmd.as_bytes())
                .unwrap();

            let mut ready = [0u8; 6];

            port.read_exact(&mut ready)
                .unwrap();

            if &ready != b"READY\n" {
                panic!(
                    "Unexpected response: {:?}",
                    ready
                );
            }

            let start = Instant::now();

            for chunk in data.chunks(65536) {
                port.write_all(chunk)
                    .unwrap();
            }

            let elapsed =
                start.elapsed().as_secs_f64();

            let mbps =
                (data.len() as f64
                    / 1024.0
                    / 1024.0)
                    / elapsed;

            println!(
                "Write complete ({:.2} MB/s)",
                mbps
            );
        }
    }
}