extern crate libc;

use std::process::{Command, Stdio};
use std::os::unix::process::CommandExt;
use std::os::unix::io::FromRawFd;
use std::fs;
use std::ptr;
use std::thread;
use std::time;

fn main() {

    let mut ptm = 0;
    let mut pts = 0;

    let ws = libc::winsize{ws_row: 24, ws_col: 80, ws_xpixel: 0, ws_ypixel: 0};

    cvt(unsafe {
        libc::openpty(&mut ptm, &mut pts, 0 as *mut libc::c_char, ptr::null() as *const libc::termios, &ws)
    }).unwrap();

    println!("openpty gave master {:?}, slave {:?}", ptm, pts);

    let mut builder = Command::new("cat");

    builder.stdin(unsafe{Stdio::from_raw_fd(pts)})
           .stdout(unsafe{Stdio::from_raw_fd(pts)})
           .stderr(unsafe{Stdio::from_raw_fd(pts)})
           .before_exec(move || {
                unsafe {
                    cvt(libc::close(ptm)).unwrap();
                    cvt(libc::close(pts)).unwrap();
                    cvt(libc::setsid()).unwrap();
                    libc::sleep(2);
                }
                //thread::sleep(time::Duration::from_millis(10000));
                printfds("child");
                Ok(())
                });

    let mut child = builder.spawn().unwrap();
    thread::sleep(time::Duration::from_millis(1000));

    cvt(unsafe {libc::close(pts) }).unwrap();
    let ptm2 = cvt(unsafe {libc::dup(ptm) }).unwrap();

    printfds("parent");

    inout_spawn(ptm, ptm2);

    println!("Hello, world! {:?}", child.id());
    child.wait().unwrap();
}

const stimuli: [u8; 6] = ['t' as u8, 'e' as u8, 's' as u8, 't' as u8, '\n' as u8, '\x04' as u8];

fn inout_spawn(input: libc::c_int, output: libc::c_int) {
    // writer
    let t1 = thread::spawn(move || {
        cvt(unsafe {libc::write(input, stimuli.as_ptr() as *const libc::c_void, stimuli.len())}).unwrap()
    });
    // reader
    let t2 = thread::spawn(move || {
        let mut buf = [0u8; 100];
        let mut len = 0;
        loop {
            let len = cvt(unsafe {
                libc::read(output, buf.as_ptr() as *mut libc::c_void, buf.len())
            }).unwrap();
            if len == 0 || len == -1 {
                break;
            }
            println!("{}", String::from_utf8(buf[0..len as usize].to_vec()).unwrap());
        }

    });

    t2.join();
    t1.join();
}

trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

impl IsMinusOne for i32 {
    fn is_minus_one(&self) -> bool { *self == -1 }
}
impl IsMinusOne for isize {
    fn is_minus_one(&self) -> bool { *self == -1 }
}

fn cvt<T: IsMinusOne>(t: T) -> std::io::Result<T> {
    use std::io;

    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}

fn printfds(prefix: &str) {
    for entry in fs::read_dir("/proc/self/fd").unwrap() {
        let entry = entry.unwrap();
        let canon_path = fs::canonicalize(entry.path());
        let path = entry.path();
        println!("{} {:?} {:?}", prefix, path, canon_path);
    }
}

