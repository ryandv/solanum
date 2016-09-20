extern crate libc;
extern crate mio;
extern crate mio_uds;
extern crate nix;
extern crate solanum;

use mio_uds::{UnixListener, UnixStream};
use mio::channel;
use mio::{ Events, Poll, PollOpt, Ready, Token };
use mio::unix::EventedFd;

use nix::libc::{c_char, chdir, exit, EXIT_FAILURE, EXIT_SUCCESS, fork, getpid, pid_t, umask, setsid};
use nix::sys;
use nix::sys::signal;

use solanum::daemon;

use std::ffi::{CString};
use std::fs;
use std::io::{Error, Read, Write};
use std::mem;
use std::net::Shutdown;
use std::os::unix::io::{FromRawFd, RawFd};
use std::path::Path;
use std::time;
use std::thread;

enum SignalMessage {
}

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

    let mut pid_file = fs::File::create(Path::new("/tmp/solanum.pid")).unwrap();
    unsafe {
        pid_file.write_fmt(format_args!("{}", getpid()));
    }
}

fn handle_acceptor(listener : &UnixListener)
{
    let (mut stream, _) = listener.accept().unwrap().unwrap();
    let mut message = String::new();
    stream.read_to_string(&mut message).unwrap();
    stream.shutdown(Shutdown::Both);
}

fn main()
{
    daemonize();

    let mut poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);

    let listener = UnixListener::bind("/tmp/solanum").unwrap();
    poll.register(&listener, Token(0), Ready::readable(), PollOpt::edge()).expect("could not register listener with poll");

    //let (sender, receiver) : (channel::Sender<SignalMessage>, channel::Receiver<SignalMessage>) = channel::channel();

    //let signalThread = thread::spawn(move || {
        //receiver.try_recv().unwrap();
    unsafe {
        let mut block_mask : libc::sigset_t = mem::uninitialized();
        let mut old_block_mask : libc::sigset_t = mem::uninitialized();
        libc::sigemptyset(&mut block_mask as *mut libc::sigset_t);
        libc::sigaddset(&mut block_mask as *mut libc::sigset_t, libc::SIGTERM);

        libc::pthread_sigmask(libc::SIG_BLOCK, &block_mask as *const libc::sigset_t, &mut old_block_mask as *mut libc::sigset_t);
        let rawfd = libc::signalfd(-1 as libc::c_int, &block_mask as *const libc::sigset_t, 0 as libc::c_int);
        let signalfd = EventedFd(&rawfd);
        poll.register(&signalfd, Token(1), Ready::readable(), PollOpt::edge()).expect("could not register signalfd with poll");
    }
    //});
    //let signals = EventedFd(signalfd::signalfd(-1, signal::SigSet::empty().add(signal::Signal::SIGTERM), signalfd::SfdFlags::from_bits(signalfd::SFD_NONBLOCK)).unwrap());

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                Token(0) => {
                    handle_acceptor(&listener);
                },
                Token(1) => {
                    println!("Signal received");
                    drop(listener);
                    fs::remove_file(Path::new("/tmp/solanum")).unwrap();
                    thread::sleep(time::Duration::new(5, 0));
                    return;
                }
                _ => {
                    panic!("Unhandled token");
                }
            }
        }
    }
}
