use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    process::Command,
    str,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Server listening on 127.0.0.1:7878");

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        handle_connection(stream);
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    println!("Raw request:\n{}", request);

    // Parse the request to find Content-Length and JSON body
    let (content_length, json_body) = parse_http_request(&request);

    // Parse JSON to get command and arguments
    let (command, arguments) = parse_json_body(&json_body);

    println!("Executing: {} {}", command, arguments);

    // Execute the command
    let response_body = match run_shell_command(&command, &arguments) {
        Ok(output) => format!("Success:\n{}", output),
        Err(e) => format!("Error: {}", e),
    };

    // Send HTTP response
    let response = format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/plain\r\n\
         Content-Length: {}\r\n\
         Connection: close\r\n\
         \r\n\
         {}",
        response_body.len(),
        response_body
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}

fn parse_http_request(request: &str) -> (usize, String) {
    let mut content_length = 0;
    let mut json_body = String::new();

    // Split request into headers and body
    if let Some(body_start) = request.find("\r\n\r\n") {
        let headers = &request[..body_start];
        json_body = request[body_start + 4..].trim().to_string();

        // Find Content-Length header
        for line in headers.lines() {
            if line.starts_with("Content-Length:") {
                if let Some(len_str) = line.split(':').nth(1) {
                    content_length = len_str.trim().parse().unwrap_or(0);
                }
            }
        }
    }

    (content_length, json_body)
}

fn parse_json_body(json_body: &str) -> (String, String) {
    // Simple JSON parsing
    let mut command = String::new();
    let mut arguments = String::new();

    if let Some(cmd_start) = json_body.find("\"command\":\"") {
        if let Some(cmd_end) = json_body[cmd_start + 11..].find('\"') {
            command = json_body[cmd_start + 11..cmd_start + 11 + cmd_end].to_string();
        }
    }

    if let Some(args_start) = json_body.find("\"arguments\":\"") {
        if let Some(args_end) = json_body[args_start + 13..].find('\"') {
            arguments = json_body[args_start + 13..args_start + 13 + args_end].to_string();
        }
    }

    (command, arguments)
}

fn run_shell_command(
    command_str: &str,
    command_args: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut command = Command::new(command_str);

    // Split the arguments string into individual arguments
    for arg in command_args.split_whitespace() {
        command.arg(arg);
    }

    let output = command.output()?;

    if output.status.success() {
        Ok(str::from_utf8(&output.stdout)?.to_string())
    } else {
        let error_msg = str::from_utf8(&output.stderr)?.to_string();
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Command failed: {}", error_msg),
        )))
    }
}
