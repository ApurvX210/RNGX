use std::{fs, io::{BufRead, BufReader, Write}, net::{TcpListener, TcpStream}, ops::IndexMut, thread, time::Duration};

use server::ThreadPool;
fn main(){
    let tcp_listner = TcpListener::bind("127.0.0.1:8082").expect("Error Occured while binding listner to port");
    let thread_pool = ThreadPool::build(4).unwrap();
    for tcp_stream in tcp_listner.incoming(){
        let stream = tcp_stream.unwrap();

        let join_Handler = thread_pool.execute(|| {
            handle_connection(stream);
        });
    }
}

fn handle_connection(mut stream: TcpStream){
    let buffer_reader = BufReader::new(&stream);
    let http_request: Vec<_> = buffer_reader.lines().map(|result|{
        result.unwrap()
    }).take_while(|line| !line.is_empty()).collect();


    let (status,filename) = match http_request.get(0) {
        Some(request_line) => match request_line.as_str() {
            "GET / HTTP/1.1" => ("HTTP/1.1 200 OK","hello.html"),
            "GET /sleep HTTP/1.1" => {
                thread::sleep(Duration::from_secs(5));
                ("HTTP/1.1 200 OK", "hello.html")
            },
            _ => ("HTTP/1.1 404 NOT FOUND", "404.html")
        },
        None => panic!("No Http Request Found")
    };

    let status_line = status;
    let content = fs::read_to_string(filename).unwrap_or_else(|_| "Error loading page".to_string());
    let length= content.len();

    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{content}");

    stream.write_all(response.as_bytes()).unwrap();
}