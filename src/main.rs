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
	println!("{:#?}", &request);

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
				]).join("\r\n");
				stream.write(response.as_bytes()).unwrap();
				stream.flush().unwrap();
			}
			None => {
				let response = "HTTP/1.1 404 NOT FOUND\r\n\r\n";
				stream.write(response.as_bytes()).unwrap();
				stream.flush().unwrap();
			}
		}
	}
}

fn process_request(request: String) -> HashMap<String,String> {
	let request_array: Vec<&str> = request.split("\r\n").collect();
	let mut request_properties: HashMap<String,String> =
		request_array
			.iter()
			.enumerate()
			.filter(|(i, _line)| i>&1) //process first line separately
			.map(|(_i,line)| { //split lines on ": " and add to a map
				let it: Vec<&str> = line.split(": ").collect();
				return (it[0].to_string(), it[1].to_string());
			})
			.collect();
	//save standard request properties separately as they are all on the same line
	let http_std_request: Vec<&str> = request_array[0].split(" ").collect();
	if http_std_request.len() == 3 {
		request_properties.insert("Method".to_string(),http_std_request[0].to_string());
		request_properties.insert("Url".to_string(),http_std_request[1].to_string());
		request_properties.insert("Http".to_string(),http_std_request[2].to_string());
	}
	return request_properties;
}
