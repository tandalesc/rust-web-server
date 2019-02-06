use std::collections::HashMap;
use std::fs;

use util;
use http;

type Request<'a> = &'a HashMap<String,String>;
type Response<'a> = String;
type RequestHandler = Fn(Request)->Response;

pub struct ControllerRule
{
    pub method: http::HttpMethod,
	pub url: &'static str,
    pub action: Box<RequestHandler>
}

//sample action that returns the contents of a file
pub fn file_response(file_name: &'static str) -> Box<Fn(Request)->Response> {
    return Box::new(move |request| {
        let mut contents = fs::read_to_string(&file_name).unwrap();
        //replace optional variables prior to calculating content length
        contents = contents.replace("{!request}", &format!("{:#?}",&request));
        let content_length = contents.chars().count();
        let mut response = HashMap::new();
        response.insert("Version".to_string(),"HTTP/1.1".to_string());
        response.insert("Status".to_string(),"200".to_string());
        response.insert("Reason".to_string(),"OK".to_string());
        response.insert("Content-Type".to_string(),"text/html".to_string());
        response.insert("Content-Length".to_string(),format!("{}",content_length).to_string());
        response.insert("Body".to_string(),contents.to_string());
        return util::process_response(&response);
    });
}

pub fn text_response(text: &'static str) -> Box<Fn(Request)->Response> {
    return Box::new(move |request| {
        let content_length = text.chars().count();
        let mut response = HashMap::new();
        response.insert("Version".to_string(),"HTTP/1.1".to_string());
        response.insert("Status".to_string(),"200".to_string());
        response.insert("Reason".to_string(),"OK".to_string());
        response.insert("Content-Type".to_string(),"text/plain".to_string());
        response.insert("Content-Length".to_string(),format!("{}",content_length).to_string());
        response.insert("Body".to_string(),text.to_string());
        return util::process_response(&response);
    });
}

//default controller
pub fn get_rules() -> Vec<ControllerRule> {
    return vec![
        ControllerRule {
            method: http::HttpMethod::GET,
            url: "/",
            action: file_response("test.html")
        },
        ControllerRule {
            method: http::HttpMethod::GET,
            url: "/blog",
            action: file_response("blog.html")
        },
        ControllerRule {
            method: http::HttpMethod::GET,
            url: "/textResTest",
            action: text_response("This is a plain-text response.")
        }
    ];
}

//get an action for a given url endpoint (or None)
pub fn match_rule(url: &str) -> Option<Box<RequestHandler>> {
    for rule in get_rules() {
        if rule.url == url {
            return Some(rule.action);
        }
    }
    return None;
}
