extern crate nix;

use std::ffi::{CString};
use std::io::Error;
use nix::libc::{c_char, chdir, exit, EXIT_FAILURE, EXIT_SUCCESS, fork, getpid, pid_t, umask, setsid};
use nix::unistd::{sleep};

fn main()
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

        println!("Daemonized");

        loop {
            println!("Hello world");
            sleep(5);
        }
    }
}
