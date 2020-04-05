use serde::{ Deserialize, Serialize };
use hyper::{ Body };
use crate::utils::{ json_error, into_internal_error };

#[derive(Debug)]
pub struct Request {
	pub method: String,
	body_len: u64,
	pub body: Option<Body>, /* or stream */
	username: Option<String> /* is the user connected */
	// TODO role
}

impl Request {
	pub fn froom(req: hyper::Request<Body>) -> Result<(String, Self), Response> {
		// TODO: Handle ascii
		let path = &req.uri().path()[1..];
		let ( module, method ) = match path.find('/') {
			Some( index ) => {
				let module = path[..index].into();
				// TODO: Remove trailing '/' if present
				let method = &path[index + 1..];
				if method.is_empty() {
					return Err(Response::new(Code::BadRequest, &json_error("Need a method name")))?;
				}
				// TODO: maybe change char '/' by '_' in method
				( module, method.into() )
			},
			None => {
				if path.is_empty() {
					( "static".into(), "index.html".into() )
				} else {
					return Err(Response::new(Code::NotFound, &json_error("Invalid method name")));
				}
			}
		};
		let username = if req.headers().contains_key("Authorization") {
			None // username from token
		} else {
			None
		};
		let ( body_len, body ) = match req.headers().get("Content-Length") {
			Some( value ) => {
				let body_len = value.to_str().map_err(into_internal_error)?.parse().map_err(into_internal_error)?;
				let body = Some(req.into_body()); // consummate the body
				( body_len, body )
			}
			None => ( 0, None )
		};
		// println!("body_len: {:?}", body_len);
		// println!("body: {:?}", body);
		Ok ((module, Request {
			method,
			body,
			body_len,
			username
		}))
	}
} 

#[allow(dead_code)]
pub enum Code {
	OK = 200,
	Created = 201,
	Accepted = 202,
	NoContent = 204,
	PartialContent = 206, // Have to send another stuff
	BadRequest = 400,
	Unauthorized = 401, // you are not connected
	Forbidden = 403, // you do not have permission but are connected
	NotFound = 404,
	MethodNotAllowed = 405,
	NotAcceptable = 406, // Cannot return accpeted-content of the request header
	TimeOut = 408,
	Conflict = 409, // Resource already exist
	LengthRequired = 411, 
	PayloadToLarge = 413,
	UnsupportedMediaType = 415,
	RangeNotSatisfiable = 416, // file index required is after EOF
	InternalServerError = 500
	
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
	code: u16,
	body_len: u64,
	body: String
}

impl Response {
	pub fn new(code: Code, no: &str) -> Self {
		Response {
			body_len: 54,
			code: code as u16,
			body: no.into()
		}
	}
}

/* impl http code conversion */

impl Into<hyper::Response<Body>> for Response {
	fn into(self) -> hyper::Response<Body> {
		hyper::Response::builder()
			.status(self.code)
			.body(Body::from(self.body)).unwrap_or(
				hyper::Response::new("Internal error when writing response".into())
			)
	}
}