use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::io::{Read, ErrorKind, Write};
use std::time::Duration;

const BIND_ADDR: &str = "127.0.0.1:3987";
const MSG_SIZE: usize = 32;


fn sleep () {
    thread::sleep(Duration::from_millis(100));
}

fn main() {
    let server = TcpListener::bind(BIND_ADDR).expect("server bind error");
    server
        .set_nonblocking(true)
        .expect("server set nonblocking error");

    let mut clients = vec![];

    let (tx, rx) = mpsc::channel::<String>();

    println!("Server Start...");

    loop {
        if let Ok((mut client, addr)) = server.accept() {
            println!("Client connected {}", addr);
            clients.push(client.try_clone().expect("failed to clone client"));
            let tx: mpsc::Sender<String> = tx.clone();

            // 将tx传递给线程闭包，后续的tx从client获取
            thread::spawn(move || loop {
                let mut buf = vec![0; MSG_SIZE];
                match client.read_exact(&mut buf) {
                    Ok(_) => {
                        let bufs = buf.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(bufs).expect("have invalid chars");
                        println!("Client {} send {}", addr, msg);
                        tx.send(msg).expect("failed to send msg to rx");
                    },
                    Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing the connection with: {}", addr);
                        break;
                    }
                }
                sleep();
            });
        }

        if let Ok(msg) = rx.try_recv() {
            clients = clients.into_iter().filter_map(|mut client| {
                let mut buff = msg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).map(|_| client).ok()
            }).collect::<Vec<_>>();
        }
        sleep();
    }
}
