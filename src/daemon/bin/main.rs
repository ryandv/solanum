#[macro_use]
extern crate log;
extern crate log4rs;
extern crate mio;
extern crate mio_uds;
extern crate nix;
extern crate solanum;

use mio::{ Events, Poll, PollOpt, Ready, Token };
use mio::unix::EventedFd;

use solanum::daemon;

use nix::libc;

use std::ffi::CString;
use std::fs;
use std::error::Error;
use std::io::{ Error as IOError, Write };
use std::mem;
use std::path::Path;
use std::time;
use std::thread;

unsafe fn daemonize()
{
    let child_pid : libc::pid_t;
    let daemon_pid : libc::pid_t;
    let sid : libc::pid_t;
    let c_root : *const libc::c_char;

    let root = CString::new("/");

    match root {
        Ok(ref s) => c_root = s.as_ptr(),
        Err(_) => libc::exit(libc::EXIT_FAILURE)
    }

    libc::umask(0);

    child_pid = libc::fork();
    if child_pid < 0 {
        println!("Could not fork child process");
        libc::exit(libc::EXIT_FAILURE);
    } else if child_pid > 0 {
        println!("Parent process ID: {}", child_pid);
        libc::exit(libc::EXIT_SUCCESS);
    }
    println!("Child process ID: {}", libc::getpid());

    sid = libc::setsid();
    if sid < 0 {
        println!("Could not setsid");
        libc::exit(libc::EXIT_FAILURE);
    }

    daemon_pid = libc::fork();
    if daemon_pid < 0 {
        println!("Could not fork grandchild process");
        libc::exit(libc::EXIT_FAILURE);
    } else if daemon_pid > 0 {
        libc::exit(libc::EXIT_SUCCESS);
    }
    println!("Grandchild process ID: {}", libc::getpid());

    let chdir_result = libc::chdir(c_root);

    if chdir_result < 0 {
        println!("Could not chdir to root: {}", IOError::last_os_error());
        libc::exit(libc::EXIT_FAILURE);
    }

    println!("Daemonized");

    let mut pid_file = fs::File::create(Path::new("/tmp/solanum.pid")).unwrap();
    pid_file.write_fmt(format_args!("{}", libc::getpid())).unwrap();
}

unsafe fn register_signalfd_poll(poll : &Poll)
{
    let mut block_mask : libc::sigset_t = mem::uninitialized();
    let mut old_block_mask : libc::sigset_t = mem::uninitialized();

    libc::sigemptyset(&mut block_mask as *mut libc::sigset_t);
    libc::sigaddset(&mut block_mask as *mut libc::sigset_t, libc::SIGTERM);
    libc::pthread_sigmask(
        libc::SIG_BLOCK,
        &block_mask as *const libc::sigset_t,
        &mut old_block_mask as *mut libc::sigset_t
    );

    let rawfd = libc::signalfd(
        -1 as libc::c_int,
        &block_mask as *const libc::sigset_t,
        0 as libc::c_int
    );
    let signalfd = EventedFd(&rawfd);
    poll.register(&signalfd, Token(1), Ready::readable(), PollOpt::edge()).expect("could not register signalfd with poll");
}

fn clean_up(command_processor : daemon::CommandProcessor)
{
    drop(command_processor);
    remove_pidfile();
}

fn remove_pidfile()
{
    fs::remove_file(Path::new("/tmp/solanum.pid")).unwrap();
}

fn main()
{
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    unsafe { daemonize(); }

    let poll = Poll::new().unwrap();
    let mut events = Events::with_capacity(1024);
    let command_processor = match daemon::CommandProcessor::new(&poll) {
        Ok(processor) => processor,
        Err(_) => {
            remove_pidfile();
            return;
        }
    };

    unsafe { register_signalfd_poll(&poll); }

    loop {
        match poll.poll(&mut events, None) {
            Ok(_) => {},
            Err(_) => {
                error!("Could not poll for events");
                clean_up(command_processor);
                return;
            }
        }

        for event in events.iter() {
            match event.token() {
                Token(0) => {
                    match command_processor.handle_acceptor() {
                        Ok(_) => {
                        },
                        Err(e) => {
                            error!("{}", e.description());
                            clean_up(command_processor);
                            return;
                        }
                    }
                },
                Token(1) => {
                    info!("Signal received");
                    clean_up(command_processor);
                    return;
                }
                _ => {
                    error!("Unhandled token received");
                    clean_up(command_processor);
                    return;
                }
            }
        }
    }
}
