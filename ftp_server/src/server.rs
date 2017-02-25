use std::io::prelude::*; //the standard io functions that come with rust
use std::collections::HashMap;
use std::io::BufReader;
use std::string::String;
use std::net::{TcpStream, SocketAddrV4};
use std::path::Path;
use std::fs;

use user::User;


pub const OPENNING_DATA_CONNECTION: u32 = 150;
pub const OPERATION_SUCCESS: u32 = 200;
pub const SYSTEM_RECEIVED: u32 = 215;
pub const LOGGED_EXPECTED: u32 = 220;
pub const GOODBYE: u32 = 221;
pub const CLOSING_DATA_CONNECTION: u32 = 226;
pub const PASSIVE_MODE: u32 = 227;
pub const LOGGED_IN: u32 = 230;
pub const CWD_CONFIRMED: u32 = 250;
pub const PATHNAME_AVAILABLE: u32 = 257;
pub const PASSWORD_EXPECTED: u32 = 331;
pub const INVALID_USER_OR_PASS: u32 = 430;
pub const NOT_UNDERSTOOD: u32 = 500;
pub const AUTHENTICATION_FAILED: u32 = 530;
pub const NO_ACCESS: u32 = 550;


#[derive(Debug, Copy, Clone)]
pub enum FtpMode {
    Active(SocketAddrV4),
    Passive,
}

pub fn write_response(client: &mut BufReader<TcpStream>, cmd: &str) {
    client.get_mut()
        .write(cmd.to_string().as_bytes())
        .expect("Something went wrong writing command");
    client.get_mut().flush().expect("Something went wrong flushing stream");
}

pub fn read_message(client: &mut BufReader<TcpStream>) -> String {
    let mut response = String::new();
    client.read_line(&mut response).expect("Could not read message");
    println!("CLIENT: {}", response);

    return response;

}

pub fn handle_user(mut client: &mut BufReader<TcpStream>,
                   arg: &str,
                   map: &HashMap<String, User>)
                   -> bool {

    match map.get(arg) {
        Some(user) => {
            write_response(client,
                           &format!("{} Username okay, need password for {}\r\n",
                                    PASSWORD_EXPECTED,
                                    arg));
            let response = read_message(&mut client);

            let line = response.trim();

            let (cmd, password) = match line.find(' ') {
                Some(pos) => (&line[0..pos], &line[pos + 1..]),
                None => (line, "".as_ref()),
            };

            match cmd {
                "PASS" | "pass" => {
                    if password.trim() == user.pass {
                        write_response(client,
                                       &format!("{} Success Login for {}\r\n", LOGGED_IN, arg));
                        return true;
                    } else {

                        write_response(client,
                                       &format!("{} Invalid Password {}\r\n",
                                                INVALID_USER_OR_PASS,
                                                arg));
                        return false;
                    }
                }
                _ => {
                    write_response(client,
                                   &format!("{} {} not understood\r\n", NOT_UNDERSTOOD, cmd));
                    return false;
                }
            }

        }
        None => {

            info!("The user does not exist");
            write_response(client,
                           &format!("{} Invalid Username {}\r\n", INVALID_USER_OR_PASS, arg));
            return false;
        }
    }
}

//TODO: fixing here after implementing ls command
pub fn cwd(client: &mut BufReader<TcpStream>, args: &str, user: &mut User) {
    println!("user path: {}", user.path);
    println!("cur path: {}", user.cur_dir);

    let cur_dir = format!("{}/{}", user.path, args);
    let user_dir = format!("{}", user.path);

    let cur_path = Path::new(&cur_dir);
    let user_root = Path::new(&user_dir);
    println!("is cur>root{}", cur_path < user_root);

    match cur_path < user_root {
        true => {
            write_response(client,
                           &format!("{} CWD Command Success \r\n", CWD_CONFIRMED));
        }
        false => {
            if cur_path.exists() {
                user.cur_dir = cur_dir.to_string();
                write_response(client,
                               &format!("{} CWD Command Success \r\n", CWD_CONFIRMED));
            } else {
                write_response(client,
                               &format!("{} {} No Such File or Directory \r\n", NO_ACCESS, args));
            }
        }
    }


}



//TODO: Role check in main function instead of here
pub fn mkd(client: &mut BufReader<TcpStream>, args: &str, user: &mut User) {

    let new_dir = format!("{}/{}", user.cur_dir, args);
    let path = Path::new(&new_dir);

    if !path.exists() {
        fs::create_dir_all(&path).expect("Could not create directory");
    }

    write_response(client,
                   &format!("{} {} creation success\r\n", PATHNAME_AVAILABLE, args));

}

//TODO: Role check in main function instead of here
pub fn handle_type(client: &mut BufReader<TcpStream>, args: &str) -> String {
    match args {
        "i" | "I" => return "BINARY".to_string(),
        "a" | "A" => return "ASCII".to_string(),
        _ => return "".to_string(),
    }
}