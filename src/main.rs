extern crate libc;
extern crate csv;
extern crate chrono;
extern crate rustc_serialize;
extern crate docopt;
extern crate xml_writer;

// That is what they effectively are:
// type c_char = i8;
// type pid_t = i32;
use libc::c_char;
use libc::pid_t;

use chrono::*;

use rustc_serialize::json;

use docopt::Docopt;

use xml_writer::XmlWriter;

use std::convert::From;
use std::default::Default;
use std::io::prelude::*;
use std::io::ErrorKind::UnexpectedEof;
use std::fs::File;
use std::mem;

extern {
    fn _exit(status: libc::c_int) -> libc::c_void;
}

#[cfg(target_arch="x86")]
fn exit(status: i32) -> ! {
    unsafe { _exit(status) };
    panic!("ahh!");
}

#[cfg(target_arch="x86_64")]
fn exit(status: i32) -> ! {
    std::process::exit(status);
}

macro_rules! println_stderr(
    ($($arg:tt)*) => { {
        let r = writeln!(&mut ::std::io::stderr(), $($arg)*);
        r.expect("failed printing to stderr");
    } }
);

const UT_LINESIZE: usize = 32;
const UT_NAMESIZE: usize = 32;
const UT_HOSTSIZE: usize = 256;

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct exit_status {
    e_termination: i16,
    e_exit: i16,
}

#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
struct ut_tv {
    tv_sec: i32,
    tv_usec: i32,
}

#[repr(C)]
struct utmp {
    ut_type: i16,
    ut_pid: pid_t,
    ut_line: [c_char; UT_LINESIZE],
    ut_id: [c_char; 4],
    ut_user: [c_char; UT_NAMESIZE],
    ut_host: [c_char; UT_HOSTSIZE],
    ut_exit: exit_status,
    ut_session: i32,
    ut_tv : ut_tv,
    ut_addr_v6: [i32; 4],
    __glibc_reserved: [c_char; 20],
}

impl Default for utmp {
    fn default() -> utmp {
        utmp {
            ut_type: Default::default(),
            ut_pid: Default::default(),
            ut_line: Default::default(),
            ut_id: Default::default(),
            ut_user: Default::default(),
            ut_host: [0; UT_HOSTSIZE],
            ut_exit: Default::default(),
            ut_session: Default::default(),
            ut_tv: Default::default(),
            ut_addr_v6: Default::default(),
            __glibc_reserved: Default::default(),
        }
    }
}

impl std::fmt::Debug for utmp {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut host = String::with_capacity(UT_HOSTSIZE);
        let mut line = String::with_capacity(UT_LINESIZE);
        let mut id = String::with_capacity(4);
        let mut user = String::with_capacity(UT_NAMESIZE);
        for x in 0..UT_HOSTSIZE {
            if self.ut_host[x] == 0 {
                break;
            }
            host.push(self.ut_host[x] as u8 as char);
        };
        for x in 0..UT_LINESIZE {
            if self.ut_line[x] == 0 {
                break;
            }
            line.push(self.ut_line[x] as u8 as char);
        };
        for x in 0..4 {
            if self.ut_id[x] == 0 {
                break;
            }
            id.push(self.ut_id[x] as u8 as char);
        };
        for x in 0..UT_HOSTSIZE {
            if self.ut_user[x] == 0 {
                break;
            }
            user.push(self.ut_user[x] as u8 as char);
        };

        write!(f, "utmp {{ ut_type: {}, ut_pid: {}, ut_line: {:?}, ut_id: {:?}, ut_user: {:?}, ut_host: {:?}, ut_exit: {:?}, ut_session: {:?}, ut_tv: {:?}, ut_addr_v6: {:?}}}",
          self.ut_type, self.ut_pid, line, id, user, host, self.ut_exit, self.ut_session, self.ut_tv, self.ut_addr_v6)
    }
}

#[derive(Debug, Clone, Copy, RustcEncodable)]
enum LogonType {
    Empty,
    RunLvl,
    BootTime,
    NewTime,
    OldTime,
    InitProcess,
    LoginProcess,
    UserProcess,
    DeadProcess,
    Accounting,
}
impl Default for LogonType {
    fn default() -> LogonType {
        LogonType::Empty
    }
}

#[derive(Debug, Default, Clone, RustcEncodable)]
struct LogonEntry {
    logon_type: LogonType,
    user: String,
    device: String,
    pid: u32,
    host: String,
    timestamp: String,
    time_epoch: u32,
    ip_addr: String,
}

impl From<utmp> for LogonEntry {
    fn from(u: utmp) -> LogonEntry {
        let mut host = String::with_capacity(UT_HOSTSIZE);
        let mut line = String::with_capacity(UT_LINESIZE);
        let mut user = String::with_capacity(UT_NAMESIZE);
        for x in 0..UT_HOSTSIZE {
            if u.ut_host[x] <= 0 {
                break;
            }
            host.push(u.ut_host[x] as u8 as char);
        };
        for x in 0..UT_LINESIZE {
            if u.ut_line[x] <= 0 {
                break;
            }
            line.push(u.ut_line[x] as u8 as char);
        };
        for x in 0..UT_HOSTSIZE {
            if u.ut_user[x] <= 0 {
                break;
            }
            user.push(u.ut_user[x] as u8 as char);
        };
        let t = UTC.timestamp(u.ut_tv.tv_sec as i64, u.ut_tv.tv_usec as u32 * 1000);

        // by default, IP is an empty string
        let mut ip = String::new();

        // if the first part of the array is > 0, but all others are not, we assume IPv4
        if u.ut_addr_v6[0] != 0 && u.ut_addr_v6[1] == 0 && u.ut_addr_v6[2] == 0 && u.ut_addr_v6[3] == 0 {
            let b: [u8; 4] = unsafe { std::mem::transmute(u.ut_addr_v6[0]) };
            ip = format!("{}.{}.{}.{}", b[0], b[1], b[2], b[3]);
        // if any of them is > 0, it must be an IPv6
        } else if u.ut_addr_v6[0] != 0 || u.ut_addr_v6[1] != 0 || u.ut_addr_v6[2] != 0 || u.ut_addr_v6[3] != 0 {
            let b: [u8; 16] = unsafe { std::mem::transmute(u.ut_addr_v6) };
            ip = format!("{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}:{:02x}{:02x}",
              b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15]);
        }

        LogonEntry {
            logon_type: match u.ut_type {
                1 => LogonType::RunLvl,
                2 => LogonType::BootTime,
                3 => LogonType::NewTime,
                4 => LogonType::OldTime,
                5 => LogonType::InitProcess,
                6 => LogonType::LoginProcess,
                7 => LogonType::UserProcess,
                8 => LogonType::DeadProcess,
                9 => LogonType::Accounting,
                _ => LogonType::Empty,
            },
            user: user,
            device: line,
            pid: u.ut_pid as u32,
            host: host,
            timestamp: t.to_rfc3339(),
            time_epoch: u.ut_tv.tv_sec as u32,
            ip_addr: ip,
        }

        //LogonEntry::default()
    }
}

fn write_xml(entries: Vec<LogonEntry>) -> Result<String, std::io::Error> {
    let mut xml = XmlWriter::new(Vec::new());
    try!(xml.dtd("utf-8"));
    try!(xml.begin_elem("LogonEntries"));
    for e in &entries {
      try!(xml.begin_elem("LogonEntry"));
        try!(xml.attr_esc("type", format!("{:?}", e.logon_type).as_str()));
        try!(xml.attr_esc("user", e.user.as_str()));
        try!(xml.attr_esc("terminal", e.device.as_str()));
        try!(xml.attr_esc("pid", format!("{}", e.pid).as_str()));
        try!(xml.attr_esc("host", e.host.as_str()));
        try!(xml.attr_esc("timestamp", e.timestamp.as_str()));
        try!(xml.attr_esc("time_epoch", format!("{}", e.time_epoch).as_str()));
        try!(xml.attr_esc("remote_ip", e.ip_addr.as_str()));
      try!(xml.end_elem());
    }
    try!(xml.write("\n"));
    try!(xml.end_elem());
    //xml.close();
    try!(xml.flush());

    let actual = xml.into_inner();
    match String::from_utf8(actual) {
        Ok(x) => Ok(x),
        Err(e) => Err(std::io::Error::new(std::io::ErrorKind::Other, format!("could not convert XML writer to string: {}", e).as_str())),
    }
    //println!("{}", std::str::from_utf8(&actual).unwrap());
}

const USAGE: &'static str = "
mhwho.

Usage:

  mhwho [options]
  mhwho (-h | --help)
  mhwho --version

Options:
  -h, --help    Show this screen.
  --version     Show version.

  -a, --all         Show all logins, otherwise only user process logon types
                    are shown.
  -j, --json        Outputs all entries as JSON array. (Default)
  -l, --json-lines  Outputs each entry as JSON object on a new line.
  -c, --csv         Outputs each entry as CSV row on a new line.
      --csv-header  If you use CSV output, it will print a header row first.
  -x, --xml         Outputs all entries as XML document.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_version: bool,
    flag_all: bool,
    flag_json: bool,
    flag_json_lines: bool,
    flag_csv: bool,
    flag_csv_header: bool,
    flag_xml: bool,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.decode())
        .unwrap_or_else(|e|
           if e.fatal() {
               println_stderr!("{}", e);
               exit(1);
           } else {
               println!("{}", e);
               exit(0);
           }          
        );

    if args.flag_version {
        println!("mhwho v0.1.0");
        exit(0);
    }

    let mut f = match File::open("/var/run/utmp") {
        Ok(f) => f,
        Err(e) => {
            println_stderr!("could not open /var/run/utmp for reading: {}", e);
            exit(1);
        },
    };

    let size = mem::size_of::<utmp>();

    let mut entries: Vec<LogonEntry> = vec![];

    loop {
        let mut b = vec![0;size];
        match f.read_exact(&mut b) {
            Ok(_) => (),
            Err(e) => {
                if e.kind() == UnexpectedEof {
                    break;
                }

                println_stderr!("error reading from /var/run/utmp: {}", e);
                exit(1);
            },
        }
        let u: *const utmp = b.as_ptr() as *const utmp;
        let ut: &utmp = unsafe { &*u };

        // Skip this one if the "all" flag is not set
        if !args.flag_all && ut.ut_type != 7 {
            continue;
        }

        let utmp: utmp = utmp {
            ut_id: ut.ut_id,
            ut_tv: ut.ut_tv,
            ut_pid: ut.ut_pid,
            ut_session: ut.ut_session,
            ut_exit: ut.ut_exit,
            __glibc_reserved: ut.__glibc_reserved,
            ut_type: ut.ut_type,
            ut_host: ut.ut_host,
            ut_user: ut.ut_user,
            ut_line: ut.ut_line,
            ut_addr_v6: ut.ut_addr_v6,
        };

        entries.push(LogonEntry::from(utmp));
    }
    drop(f);

    // JSON array output
    if args.flag_json {
        println!("{}", json::encode(&entries).unwrap_or_else(|_| String::new()));
        exit(0);
    }

    // JSON lines output
    if args.flag_json_lines {
        for e in &entries {
            println!("{}", json::encode(e).unwrap_or_else(|_| String::new()));
        }
        exit(0);
    }

    // CSV output
    if args.flag_csv {
        if args.flag_csv_header {
            println!("LogonType,User,TerminalDevice,PID,Host,Timestamp,TimestampEpoch,RemoteIP");
        }
        let mut wtr = csv::Writer::from_memory();
        for e in &entries {
            match wtr.encode(e) {
                Ok(_) => (),
                Err(e) => { println_stderr!("WARNING: could not encode entry as CSV: {}", e); },
            }
        }
        print!("{}", wtr.as_string());
        exit(0);
    }

    // XML output
    if args.flag_xml {
        match write_xml(entries) {
            Ok(x) => {
                println!("{}", x);
                exit(0);
            },
            Err(e) => {
                println_stderr!("could not write XML document: {}", e);
                exit(1);
            }
        }
    }

    // Console output
    println!("{: <10.10}  {: <16.16}  {: <6.6}  {: <6.6}  {: <15.15}  {: >17.17}", "LOGON TYPE", "USER", "TTY", "PID", "HOST", "LOGIN@");
    for e in &entries {
        let t = UTC.timestamp(e.time_epoch as i64, 0);
        println!("{: <10.10}  {: <16.16}  {: <6.6}  {: <6.6}  {: <15.15}  {: <17.17}",
            format!("{:?}", e.logon_type), e.user, e.device, e.pid, e.host, t.format("%v %R") );
    }
    exit(0);
}
