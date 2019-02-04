use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;
use std::fs;
use std::collections::HashMap;

static HOST: &'static str = "127.0.0.1";
static PORT: &'static str = "7878";

fn main() {
	let listener = TcpListener::bind(format!("{}:{}",HOST,PORT)).unwrap();
	println!("Listening on {}:{}",HOST,PORT);

	let get_mappings = vec![
		("/".to_string(), "test.html".to_string()),
		("/blog".to_string(), "blog.html".to_string())
	];

	let get_mappings: HashMap<_,_> = get_mappings.into_iter().collect();

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		handle_connection(stream, &get_mappings);
	}
}

fn handle_connection(mut stream: TcpStream, get_mappings: &HashMap<String,String>) {
	let mut buffer = [0; 512];
	stream.read(&mut buffer).unwrap();

	let request_str = String::from_utf8_lossy(&buffer[..]).to_string();
	let request = process_request(request_str);

	if request.get("Method").unwrap() == "GET" {
		let actual_req_url = request.get("Url").unwrap();
		let search = get_mappings.keys().find(|url| url==&actual_req_url);
		match search {
			Some(request_url) => {
				let file_name = get_mappings.get(request_url).unwrap();
				let contents = fs::read_to_string(file_name).unwrap();
				let content_length_header = format!("Content-Length: {}", contents.chars().count());
				let response = (vec![
					"HTTP/1.1 200 OK",
					"Content-Type: text/html",
					&content_length_header,
					"",
					&contents
				]).join("\n");
				stream.write(response.as_bytes()).unwrap();
				stream.flush().unwrap();
			}
			None => {
				let response = "HTTP/1.1 404 NOT FOUND\n\n";
				stream.write(response.as_bytes()).unwrap();
				stream.flush().unwrap();
			}
		}
	}
}

fn process_request(request: String) -> HashMap<String,String> {
	let mut request_properties: HashMap<String,String> =
		request
			.split("\n")
			.enumerate()
			.filter(|(i,_line)| i>&1) //process first line separately
			.map(|(_i,line)| { //split lines on ": " and add to a map
				let mut it = line.split(": ");
				return (it.next().unwrap().to_string(), it.next().unwrap().to_string());
			})
			.collect();
	//save standard request properties separately as they are all on the same line
	let first_line = request.split("\n").next().unwrap();
	request_properties.insert("Request".to_string(),first_line.to_string());
	let mut http_std_request = first_line.split(" ");
	request_properties.insert("Method".to_string(),http_std_request.next().unwrap().to_string());
	request_properties.insert("Url".to_string(),http_std_request.next().unwrap().to_string());
	request_properties.insert("Http".to_string(),http_std_request.next().unwrap().to_string());
	return request_properties;
}
