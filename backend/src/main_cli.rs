use backend::library::handle_request::*;


fn main() {
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
                eprintln!("{}", msg);
                eprintln!("Done \"{}\" in {} ms", command_str, started.elapsed().as_millis())
            },
            Err(msg) => eprintln!("{}", msg),
        }
    }
}
