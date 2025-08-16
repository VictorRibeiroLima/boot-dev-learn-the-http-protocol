use std::{
    io::Read,
    net::{TcpListener, TcpStream},
    sync::mpsc,
    thread,
};

fn handle_read<R: Read + Send + 'static>(mut r: R) -> mpsc::Receiver<String> {
    let (s, rec) = mpsc::channel();

    thread::spawn(move || {
        let mut bytes: [u8; 8] = [0; 8];
        let mut current_line = String::new();
        loop {
            let read = r.read(&mut bytes).unwrap();
            let current_part = String::from_utf8_lossy(&bytes[..read]);
            let split: Vec<&str> = current_part.split("\n").collect();
            current_line.push_str(split.get(0).unwrap());

            let split_len = split.len();

            for part in &split[1..split_len] {
                if current_line.len() > 0 {
                    s.send(current_line).unwrap();
                }
                current_line = part.to_string();
            }

            if read != 8 {
                break;
            }
        }
        if current_line.len() > 0 {
            s.send(current_line).unwrap();
        }
    });

    return rec;
}

fn handle_connection(stream: TcpStream) {
    let receiver = handle_read(stream);

    loop {
        let data = receiver.recv();
        match data {
            Ok(data) => println!("read: {}", data),
            Err(_) => break,
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:42069").expect("To open the tcp");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_connection(stream);
            }
            Err(err) => {
                eprintln!("{}", err);
                panic!()
            }
        }
    }
}
