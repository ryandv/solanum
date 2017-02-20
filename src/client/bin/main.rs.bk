extern crate solanum;

use solanum::client::Client;

use std::env;
use std::process;

fn main() {
    let mut args = env::args();
    if args.len() < 2 {
        println!(
            "USAGE: solanum COMMAND

  Where COMMAND is one of:
      start : Start a new pomodoro");
        process::exit(1);
    }

    let client = Client::new();
    args.next();
    let command = args.next().expect("command not specified");

    println!("{}", client.send_message(command).unwrap());
}
