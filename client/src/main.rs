use rand::{self, Rng};
use std::io::{self, ErrorKind, Read, Write};
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::mpsc::TryRecvError;
use std::thread;
use std::time::Duration;

const LOCAL_ADDR: &str = "127.0.0.1:3987";
const BUFFER_SIZE: usize = 32;

fn main() {
    let rng = rand::thread_rng();
    let uuid: String = rng
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();

    let username = format!("User_{}", uuid);

    let mut client = TcpStream::connect(LOCAL_ADDR).expect("Cannot Connect To Server");
    client
        .set_nonblocking(true)
        .expect("Fail To Set Nonblocking");

    let (tx, rx) = mpsc::channel::<String>();

    println!("Your Name is {}", username);

    thread::spawn(move || loop {
        let mut buf = vec![0; BUFFER_SIZE];

        match client.read_exact(&mut buf) {
            Ok(_) => {
                let msg = buf.into_iter().take_while(|x| *x != 0).collect::<Vec<_>>();
                let result = String::from_utf8_lossy(&msg);
                println!("[Server Message] {}", result);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Server Disconenction");
                break;
            }
        }

        match rx.try_recv() {
            Ok(msg) => {
                let mut buff = msg.clone().into_bytes();
                // 截取前32个buffer
                buff.resize(BUFFER_SIZE, 0);
                let info_buff = username.clone() + ": ";
                client
                    .write_all(&info_buff.as_bytes())
                    .expect("Send Server Error");
                client.write_all(&buff).expect("Send Server Error");
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                println!("Server Disconnection");
                break;
            }
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write A Message:");

    loop {
        let mut buff = String::new();
        io::stdin()
            .read_line(&mut buff)
            .expect("Terminal Input Error");
        let msg = buff.trim().to_string();
        if msg == ":quit" || tx.send(msg).is_err() {
            break;
        }
    }

    println!("Good Bye!");
}
