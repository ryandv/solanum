#[macro_use]
extern crate log;
extern crate log4rs;
extern crate nix;
extern crate solanum;

use solanum::daemon;

use nix::libc;

use std::ffi::CString;
use std::fs;
use std::io;
use std::io::Write;
use std::mem;
use std::os::unix::io::RawFd;
use std::path::Path;

unsafe fn daemonize() {
    let child_pid: libc::pid_t;
    let daemon_pid: libc::pid_t;
    let sid: libc::pid_t;
    let c_root: *const libc::c_char;

    let root = CString::new("/");

    match root {
        Ok(ref s) => c_root = s.as_ptr(),
        Err(_) => libc::exit(libc::EXIT_FAILURE),
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
        println!("Could not chdir to root: {}", io::Error::last_os_error());
        libc::exit(libc::EXIT_FAILURE);
    }

    println!("Daemonized");

    libc::close(libc::STDIN_FILENO);
    libc::close(libc::STDOUT_FILENO);
    libc::close(libc::STDERR_FILENO);

    let mut pid_file = fs::File::create(Path::new("/tmp/solanum.pid")).unwrap();
    pid_file.write_fmt(format_args!("{}", libc::getpid())).unwrap();
}

unsafe fn open_signalfd<'a>() -> RawFd {
    let mut block_mask: libc::sigset_t = mem::uninitialized();
    let mut old_block_mask: libc::sigset_t = mem::uninitialized();

    libc::sigemptyset(&mut block_mask as *mut libc::sigset_t);
    libc::sigaddset(&mut block_mask as *mut libc::sigset_t, libc::SIGTERM);
    libc::pthread_sigmask(libc::SIG_BLOCK,
                          &block_mask as *const libc::sigset_t,
                          &mut old_block_mask as *mut libc::sigset_t);

    libc::signalfd(-1 as libc::c_int,
                   &block_mask as *const libc::sigset_t,
                   0 as libc::c_int)
}

fn start_daemon_container<'a>(signalfd: RawFd) -> daemon::result::Result<()> {
    let mut container = try!(daemon::DaemonContainer::new(&signalfd));
    container.start()
}

fn main() {
    // TODO: currently here because daemon chdir's to / and don't want to resolve the relative path
    // at the moment.
    let _ = log4rs::init_file("config/log4rs.yaml", Default::default()).or_else(|e| {
        error!("{}", e);
        Err(e)
    });

    let signalfd: RawFd;

    unsafe {
        daemonize();
        signalfd = open_signalfd();
    }

    let _ = start_daemon_container(signalfd).or_else(|e| {
        error!("{}", e);
        Err(e)
    });

    let _ = fs::remove_file(Path::new("/tmp/solanum.pid")).or_else(|e| {
        error!("{}", e);
        Err(e)
    });
}
