extern crate mio;
extern crate mio_uds;
extern crate nix;
extern crate solanum;

use mio_uds::{UnixListener, UnixStream};
use mio::{ Events, Poll, PollOpt, Ready, Token };

use nix::libc::{c_char, chdir, exit, EXIT_FAILURE, EXIT_SUCCESS, fork, getpid, pid_t, umask, setsid};

use solanum::daemon;

use std::ffi::{CString};
use std::io::{Error, Read};
use std::net::Shutdown;

fn daemonize()
{
    unsafe {
        let child_pid : pid_t;
        let daemon_pid : pid_t;
        let sid : pid_t;
        let c_root : *const c_char;

        let root = CString::new("/");

        match root {
            Ok(ref s) => c_root = s.as_ptr(),
            Err(_) => exit(EXIT_FAILURE)
        }

        umask(0);

        child_pid = fork();
        if child_pid < 0 {
            println!("Could not fork child process");
            exit(EXIT_FAILURE);
        } else if child_pid > 0 {
            println!("Parent process ID: {}", child_pid);
            exit(EXIT_SUCCESS);
        }
        println!("Child process ID: {}", getpid());

        sid = setsid();
        if sid < 0 {
            println!("Could not setsid");
            exit(EXIT_FAILURE);
        }

        daemon_pid = fork();
        if daemon_pid < 0 {
            println!("Could not fork grandchild process");
            exit(EXIT_FAILURE);
        } else if daemon_pid > 0 {
            exit(EXIT_SUCCESS);
        }
        println!("Grandchild process ID: {}", getpid());

        let chdir_result = chdir(c_root);

        if chdir_result < 0 {
            println!("Could not chdir to root: {}", Error::last_os_error());
            exit(EXIT_FAILURE);
        }
    }

    println!("Daemonized");
}

fn main()
{
    daemonize();
    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    let listener = UnixListener::bind("/tmp/solanum").unwrap();
    poll.register(&listener, Token(0), Ready::readable(), PollOpt::edge()).expect("could not register listener with poll");
    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                Token(0) => {
                    let (mut stream, _) = listener.accept().unwrap().unwrap();
                    let mut message = String::new();
                    stream.read_to_string(&mut message).unwrap();
                    stream.shutdown(Shutdown::Both);
                },
                _ => {
                    panic!("Unhandled token");
                }
            }
        }
    }
}
