use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fmt::Debug;
use std::fs;
use std::collections::HashMap;

static HOST: &'static str = "127.0.0.1";
static PORT: &'static str = "7878";
const DEBUG: bool = false;

fn debug<T: Debug>(o: T) {
	if DEBUG {
		println!("{:?}", o);
	}
}

fn main() {
	let host_domain = format!("{}:{}",HOST,PORT);
	let listener = TcpListener::bind(&host_domain).unwrap();
	println!("Listening on {}",&host_domain);

	let get_mappings: HashMap<_,_> = (vec![
		("/".to_string(), "test.html".to_string()),
		("/blog".to_string(), "blog.html".to_string())
	]).into_iter().collect();

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		handle_connection(stream, &get_mappings);
	}
}

//negiotiate an individual TcpStream, given a collection of mappings
fn handle_connection(mut stream: TcpStream, get_mappings: &HashMap<String,String>) {
	let mut buffer = [0; 512];
	stream.read(&mut buffer).unwrap();

	let request_str = String::from_utf8_lossy(&buffer[..]).to_string();
	let request = process_request(&request_str);
	debug(&request);

	let method = request.get("Method").unwrap();
	let url = request.get("Url").unwrap();
	//match on method first, and then url
	if method == "GET" {
		match get_mappings.get(url) {
			Some(file_name) => {
				let mut contents = fs::read_to_string(file_name).unwrap();
				//replace optional variables prior to calculating content length
				contents = contents.replace("{!request}", &format!("{:#?}",&request));
				let mut content_length = contents.chars().count();
				let response = (vec![
					"HTTP/1.1 200 OK",
					"Content-Type: text/html",
					&format!("Content-Length: {}", content_length),
					"",
					&contents
				]).join("\r\n");
				stream.write(response.as_bytes()).unwrap();
				stream.flush().unwrap();
			}
			None => {
				debug(format!("Could not find requested page {}",url));
				let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
				stream.write(response.as_bytes()).unwrap();
				stream.flush().unwrap();
			}
		}
	} else if method == "POST" {
		//TODO
		debug(format!("{} method not yet supported", method));
	} else {
		debug(format!("{} method not yet supported", method));
	}
}

//process http header into HashMap
fn process_request(request: &String) -> HashMap<String,String> {
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
