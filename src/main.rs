use std::{env, fs::{self}, io::{self, BufRead, BufReader, Write}, string::FromUtf8Error, thread, time::Duration};
use thiserror::Error;
use log::{error, info};
use tokio::net::{TcpListener,TcpStream};

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid UTF-8: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("Malformed HTTP request")]
    BadRequest,

    #[error("Internal server error")]
    Internal,
}

fn send_http_error(mut stream: TcpStream,status: &str){
    let content = fs::read_to_string("500.html").unwrap();
    let content_length = content.len();
    let response = format!("{status}\r\nContent-Length: {content_length}\r\n\r\n{content}");
    let _ = stream.write_all(response.as_bytes());
}

use server::ThreadPool;
#[tokio::main]
async fn main(){
    let tcp_listner = TcpListener::bind("127.0.0.1:8082").await.expect("Error occured while binding to the port");
    let thread_count= match env::var("NODE_COUNT") {
        Ok(val) => {
            info!("{} no of thread configured from env",val);
            print!("{}",val);
            val.parse::<usize>().unwrap_or(4)
        },
        Err(_err) => {
            error!("Node count not configured properly");
            4
        }
    };

    let thread_pool = ThreadPool::build(thread_count).unwrap();
    for tcp_stream in tcp_listner.{
        let stream = tcp_stream.unwrap();
        let error_stream = stream.try_clone().unwrap();
        thread_pool.execute(|| {
            match handle_connection(stream){
                Ok(http_request) => info!("{} handled successfully",http_request),
                Err(e) => {
                    error!("Error occured while handling request:{}",e);
                    send_http_error(error_stream,"HTTP/1.1 500 INTERNAL SERVER ERROR")
                }
            };
        });
    }
}

fn handle_connection(mut stream: TcpStream) -> Result<String, ServerError>{
    let buffer_reader = BufReader::new(&stream);
    let http_request  = match buffer_reader.lines().next(){
        Some(request_line) => request_line?,
        None => return Err(ServerError::BadRequest),
    };

    let (status,filename) = match &http_request[..] {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK","hello.html"),
        "GET /sleep HTTP/1.1" => {
            thread::sleep(Duration::from_secs(5));
            ("HTTP/1.1 200 OK", "hello.html")
        },
        _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
    };
    

    let status_line = status;
    let content = fs::read_to_string(filename)?;
    let length= content.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

    stream.write_all(response.as_bytes())?;

    Ok(http_request)
}