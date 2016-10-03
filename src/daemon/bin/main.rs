extern crate libc;
extern crate mio;
extern crate mio_uds;
extern crate nix;
extern crate solanum;

use mio::{ Events, Poll, PollOpt, Ready, Token };
use mio::unix::EventedFd;

use nix::libc::{c_char, chdir, exit, EXIT_FAILURE, EXIT_SUCCESS, fork, getpid, pid_t, umask, setsid};

use solanum::daemon;

use std::ffi::{CString};
use std::fs;
use std::io::{Error, Read, Write};
use std::mem;
use std::path::Path;
use std::time;
use std::thread;

unsafe fn daemonize()
{
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

    println!("Daemonized");

    let mut pid_file = fs::File::create(Path::new("/tmp/solanum.pid")).unwrap();
    pid_file.write_fmt(format_args!("{}", getpid())).unwrap();
}

unsafe fn register_signalfd_poll(poll : &Poll)
{
    let mut block_mask : libc::sigset_t = mem::uninitialized();
    let mut old_block_mask : libc::sigset_t = mem::uninitialized();

    libc::sigemptyset(&mut block_mask as *mut libc::sigset_t);
    libc::sigaddset(&mut block_mask as *mut libc::sigset_t, libc::SIGTERM);
    libc::pthread_sigmask(libc::SIG_BLOCK, &block_mask as *const libc::sigset_t, &mut old_block_mask as *mut libc::sigset_t);

    let rawfd = libc::signalfd(-1 as libc::c_int, &block_mask as *const libc::sigset_t, 0 as libc::c_int);
    let signalfd = EventedFd(&rawfd);
    poll.register(&signalfd, Token(1), Ready::readable(), PollOpt::edge()).expect("could not register signalfd with poll");
}

fn main()
{
    unsafe { daemonize(); }

    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    let daemon = daemon::CommandProcessor::new(&poll);

    unsafe { register_signalfd_poll(&poll); }

    loop {
        poll.poll(&mut events, None).unwrap();

        for event in events.iter() {
            match event.token() {
                Token(0) => {
                    daemon.handle_acceptor();
                },
                Token(1) => {
                    println!("Signal received");
                    drop(daemon);
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
