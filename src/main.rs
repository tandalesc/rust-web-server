use std::io::prelude::*;
use std::net::TcpStream;
use std::net::TcpListener;

mod util;
mod controller;
mod http;

static HOST: &'static str = "127.0.0.1";
static PORT: &'static str = "7878";

fn main() {
	let host_domain = format!("{}:{}",HOST,PORT);
	let listener = TcpListener::bind(&host_domain).unwrap();
	println!("Listening on {}",&host_domain);

	for stream in listener.incoming() {
		let stream = stream.unwrap();
		handle_connection(stream);
	}
}

//negiotiate an individual TcpStream, given a collection of mappings
fn handle_connection(mut stream: TcpStream) {
	let request_str = util::get_request_from_stream(&stream);
	let request = util::process_request(&request_str);
	util::debug(&request);
	let response_str: String;

	//look for Url in request
	match request.get("Url") {
		Some(url) =>
			//match the Url with our controller rules
			match controller::match_rule(url) {
				Some(rule_action) => {
					response_str = rule_action(&request).to_string();
				}
				None => {
					util::debug(format!("Could not find requested page {}",url));
					response_str = "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string();
				}
			}
		None => {
			util::debug("There was an error processing the request.");
			response_str = "HTTP/1.1 404 NOT FOUND\r\n\r\n".to_string();
		}
	}

	stream.write(response_str.as_bytes()).unwrap();
	stream.flush().unwrap();
}
