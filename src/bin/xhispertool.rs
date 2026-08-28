// xhispertool - combined daemon and client for text input via uinput.
// Port of xhispertool.c. Mode is selected by argv[0] (the "xhispertoold"
// symlink installed by packaging triggers daemon mode) or an explicit
// `--daemon` flag, exactly as in the original.

use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use xhisper_core::input::uinput::{self, XhisperDevice};
use xhisper_core::ipc::{self, Command};

const LAYOUT_FILE: &str = "/tmp/xhispertoold.layout";

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let prog = args
        .first()
        .and_then(|p| {
            Path::new(p)
                .file_name()
                .map(|f| f.to_string_lossy().into_owned())
        })
        .unwrap_or_default();

    let is_daemon = prog == "xhispertoold" || args.get(1).map(String::as_str) == Some("--daemon");

    if is_daemon {
        run_daemon()
    } else {
        run_client(&args)
    }
}

fn run_daemon() -> ExitCode {
    let mut device = match XhisperDevice::create() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("failed to open /dev/uinput: {e}");
            return ExitCode::FAILURE;
        }
    };

    let socket = match ipc::bind_daemon_socket() {
        Ok(s) => s,
        Err(e) => {
            if e.kind() == std::io::ErrorKind::AddrInUse {
                eprintln!("xhispertoold is already running");
            } else {
                eprintln!("failed to bind socket: {e}");
            }
            return ExitCode::FAILURE;
        }
    };

    let layout = env::var("XHISPER_LAYOUT")
        .ok()
        .filter(|l| !l.is_empty())
        .unwrap_or_else(|| "us".to_string());

    // Persist the layout so xhisper.sh can detect and restart a stale daemon.
    if let Ok(mut f) = fs::File::create(LAYOUT_FILE) {
        let _ = writeln!(f, "{layout}");
    }

    println!(
        "xhispertoold: listening on {} (layout: {layout})",
        ipc::socket_path().display()
    );

    let mut buf = [0u8; 2];
    loop {
        let n = match socket.recv(&mut buf) {
            Ok(n) => n,
            Err(_) => continue,
        };
        if n == 0 {
            continue;
        }
        match Command::decode(&buf[..n]) {
            Some(Command::Paste) => device.do_paste(),
            Some(Command::Type(c)) => device.type_char(c, &layout),
            Some(Command::Backspace) => device.do_backspace(),
            Some(Command::RightAlt) => device.do_key(uinput::KEY_RIGHTALT),
            Some(Command::LeftAlt) => device.do_key(uinput::KEY_LEFTALT),
            Some(Command::LeftCtrl) => device.do_key(uinput::KEY_LEFTCTRL),
            Some(Command::RightCtrl) => device.do_key(uinput::KEY_RIGHTCTRL),
            Some(Command::LeftShift) => device.do_key(uinput::KEY_LEFTSHIFT),
            Some(Command::RightShift) => device.do_key(uinput::KEY_RIGHTSHIFT),
            Some(Command::Super) => device.do_key(uinput::KEY_LEFTMETA),
            None => {}
        }
    }
}

fn show_usage() {
    eprintln!(
        "Usage:\n\
         \x20 xhispertool paste            - Paste from clipboard (Ctrl+V)\n\
         \x20 xhispertool type <char>      - Type a single ASCII character\n\
         \x20 xhispertool backspace        - Press backspace\n\
         \n\
         Input switching keys:\n\
         \x20 xhispertool leftalt          - Press left alt\n\
         \x20 xhispertool rightalt         - Press right alt\n\
         \x20 xhispertool leftctrl         - Press left ctrl\n\
         \x20 xhispertool rightctrl        - Press right ctrl\n\
         \x20 xhispertool leftshift        - Press left shift\n\
         \x20 xhispertool rightshift       - Press right shift\n\
         \x20 xhispertool super            - Press super (Windows key)\n\
         \n\
         Daemon:\n\
         \x20 xhispertoold                 - Run daemon (or xhispertool --daemon)"
    );
}

fn run_client(args: &[String]) -> ExitCode {
    if args.len() < 2 {
        show_usage();
        return ExitCode::FAILURE;
    }

    let command = match args[1].as_str() {
        "paste" => Command::Paste,
        "backspace" => Command::Backspace,
        "rightalt" => Command::RightAlt,
        "leftalt" => Command::LeftAlt,
        "leftctrl" => Command::LeftCtrl,
        "rightctrl" => Command::RightCtrl,
        "leftshift" => Command::LeftShift,
        "rightshift" => Command::RightShift,
        "super" => Command::Super,
        "type" => {
            if args.len() != 3 || args[2].len() != 1 {
                eprintln!("Error: 'type' requires exactly one character argument");
                show_usage();
                return ExitCode::FAILURE;
            }
            Command::Type(args[2].as_bytes()[0])
        }
        other => {
            eprintln!("Error: Unknown command '{other}'");
            show_usage();
            return ExitCode::FAILURE;
        }
    };

    let socket = match ipc::connect_client() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("failed to connect to xhispertoold: {e}");
            match e.raw_os_error() {
                Some(libc::ENOENT) | Some(libc::ECONNREFUSED) => {
                    eprintln!("Please check if xhispertoold is running.");
                    eprintln!("Start it with: xhispertoold &");
                }
                Some(libc::EACCES) | Some(libc::EPERM) => {
                    eprintln!("Permission denied. Check socket permissions.");
                }
                _ => {}
            }
            return ExitCode::from(2);
        }
    };

    let encoded = command.encode();
    let len = command.encoded_len();
    if let Err(e) = socket.send(&encoded[..len]) {
        eprintln!("failed to send command: {e}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
