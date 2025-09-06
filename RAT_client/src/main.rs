use std::{
    io::{self, prelude::*, BufReader},
    net::TcpStream,
    process::Command,
    str,
};

fn main() {
    // Start the server (you might want to run this in a separate process)
    println!("Connecting to server at 127.0.0.1:7878...");

    // Client interaction loop
    loop {
        println!("\n=== Command Client ===");
        println!("1. Enter command to execute (format: 'command args')");
        println!("2. Type 'exit' to quit");
        println!("3. Type 'help' for examples");
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("help") {
            println!("Examples:");
            println!("  ls -la");
            println!("  pwd");
            println!("  echo 'hello world'");
            println!("  date");
            continue;
        }

        if input.is_empty() {
            continue;
        }

        // Send command to server
        match send_command_to_server(input) {
            Ok(response) => {
                println!("Server response:");
                println!("{}", response);
            }
            Err(e) => {
                eprintln!("Error communicating with server: {}", e);
            }
        }
    }
}

fn send_command_to_server(command_input: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Parse the input into command and args
    let mut parts = command_input.splitn(2, ' ');
    let command = parts.next().unwrap_or("").trim();
    let args = parts.next().unwrap_or("").trim();

    if command.is_empty() {
        return Err("No command provided".into());
    }

    // Connect to server
    let mut stream = TcpStream::connect("127.0.0.1:7878")?;

    // Create JSON payload
    let json_payload = format!(
        "{{\"command\":\"{}\", \"arguments\":\"{}\"}}",
        command, args
    );

    // Format the HTTP request
    let request = format!(
        "POST /execute HTTP/1.1\r\n\
         Host: 127.0.0.1:7878\r\n\
         Content-Type: application/json\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        json_payload.len(),
        json_payload
    );

    // Send request
    stream.write_all(request.as_bytes())?;

    // Read full response (headers + body)
    let buf_reader = BufReader::new(&stream);
    let mut response_lines = Vec::new();
    let mut in_body = false;
    let mut body = String::new();

    for line in buf_reader.lines() {
        let line = line?;

        if line.is_empty() {
            in_body = true;
            continue;
        }

        if in_body {
            body.push_str(&line);
            body.push('\n');
        } else {
            response_lines.push(line);
        }
    }

    Ok(body.trim().to_string())
}
