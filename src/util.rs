use std::io::prelude::*;
use std::net::TcpStream;
use std::collections::HashMap;
use std::fmt::Debug;

pub const DEBUG: bool = false;

pub fn debug<T: Debug>(o: T) {
	if DEBUG {
		println!("{:?}", o);
	}
}

//process HashMap into response string
pub fn process_response(response: &HashMap<String,String>) -> String {
    let version = response.get("Version").unwrap();
    let status = response.get("Status").unwrap();
    let reason = response.get("Reason").unwrap();
    let mut response_str = format!("{} {} {}\r\n", version, status, reason);
    for (key, value) in response {
        //these keys are handled specially
        if key!="Version" && key!="Status" && key!="Reason" && key!="Body" {
            response_str.push_str(&format!("{}: {}\r\n", key, value));
        }
    }
    response_str.push_str("\r\n");
    match response.get("Body") {
        Some(body) => {
            response_str.push_str(&format!("{}",body));
        }
        None => {}
    }
    return response_str;
}

//process http header into HashMap
pub fn process_request(request: &String) -> HashMap<String,String> {
	let request_array: Vec<&str> = request.trim().split("\r\n").collect();
	let mut body_buffer = Vec::new();
	let mut is_processing_body = false;
	let mut is_processing_header = true;
	let mut request_properties = HashMap::new();
	//iterate through request lines
	for (line_num, line) in request_array.iter().enumerate() {
		if line_num == 0 { //handle the request line separately
			let request_line_parts: Vec<&str> = line.split(" ").collect();
			if request_line_parts.len() == 3 {
				request_properties.insert("Method".to_string(),request_line_parts[0].to_string());
				request_properties.insert("Url".to_string(),request_line_parts[1].to_string());
				request_properties.insert("Version".to_string(),request_line_parts[2].to_string());
			}
		} else if is_processing_header {
			let it: Vec<&str> = line.split(": ").collect();
			if it.len() == 2 { //sometimes headers fail to parse properly
				request_properties.insert(it[0].to_string(), it[1].to_string());
			} else {
				debug(format!("Parsing header failed at line:\n\n{}\n\nFull request:\n{}\n",line, request));
			}
		} else if is_processing_body {
			body_buffer.push(line.to_string());
			request_properties.insert("Body".to_string(),body_buffer.join(""));
		} else {
			//switch modes on blank line
			if line == &"" {
				is_processing_body = true;
				is_processing_header = false;
			}
		}
	}
	return request_properties;
}

//converts a stream into a string using a buffer
pub fn get_request_from_stream(mut stream: &TcpStream) -> String {
    let mut buffer = [0; 512];
	stream.read(&mut buffer).unwrap();
	return String::from_utf8_lossy(&buffer[..]).to_string();
}
