use std::{io::Write, process::ExitCode};
use common::util::eprintln_memory_usage;
use backend::library::handle_request::*;


// macro_rules! eprintln_json_items {
//     ($items: expr) => {
//         $items.into_iter().for_each(|f| {
//             serde_json::to_writer(std::io::stderr(), &f).unwrap();
//             eprintln!();
//         })
//     };
// }

fn main() -> ExitCode {
    eprintln_memory_usage();

    let cmd_arg = std::env::args().nth(1);
    if let Some(cmd_arg_str) = cmd_arg {
        let response = handle_request(cmd_arg_str, false);
        match response {
            Ok(msg) => {
                std::io::stdout().write(msg.as_bytes()).unwrap();
                std::io::stdout().write(b"\n").unwrap();
                std::io::stdout().flush().unwrap();
                return ExitCode::SUCCESS;
            },
            Err(msg) => {
                eprintln!("{}", msg);
                return ExitCode::FAILURE;
            },
        }
    }

    eprintln!("Enter city name to search by name, or id to search by climate; or use json messages");

    loop {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).unwrap();

        let command_str = buf.trim();
        if command_str.is_empty() {
            continue;
        }

        let started = std::time::Instant::now();

        let response = handle_request(command_str.into(), true);
        match response {
            Ok(msg) => {
                eprintln!("{}\n", msg);
                eprintln!("Done \"{}\" in {} ms", command_str, started.elapsed().as_millis())
            },
            Err(msg) => eprintln!("{}", msg),
        }
    }
}
