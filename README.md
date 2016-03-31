# mhwho
Show who is logged on.

Yes, this utility is doing exactly the same as our traditional **w**, **who** and **pinky** commands. But have you ever tried to parse the output of one of these commands? **w** is still your best bet for that, all the other commands all have their downsides somehow. But also with **w**, you have the issue that the username gets cut off (unless you run a newer version and you can set an environment variable to make it longer). I had enough of it, and decided to write this tool.

All it does is reading the `/var/run/utmp` file (assuming glibc format), and displaying it in the way you want to.

## Usage

See the help message as shown by the program:
```
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
```

## Examples

Here is the standard console output (which is hard-coded aligned to 80 characters per row):
```
LOGON TYPE  USER              TTY     PID     HOST                        LOGIN@
UserProces  vagrant           pts/0   2776    10.0.2.2         31-Mar-2016 16:40
UserProces  vagrant           pts/1   2863    10.0.2.2         31-Mar-2016 16:44
UserProces  vagrant           pts/2   2895    fe80::a00:27ff:  31-Mar-2016 16:45
UserProces  vagrant           pts/3   2925    fe80::a00:27ff:  31-Mar-2016 16:47
```

Here is some JSON output:
```
[{"logon_type":"UserProcess","user":"vagrant","device":"pts/0","pid":2776,"host":"10.0.2.2","timestamp":"2016-03-31T16:40:47.513326+00:00","time_epoch":1459442447,"ip_addr":"10.0.2.2"},{"logon_type":"UserProcess","user":"vagrant","device":"pts/1","pid":2863,"host":"10.0.2.2","timestamp":"2016-03-31T16:44:55.261504+00:00","time_epoch":1459442695,"ip_addr":"10.0.2.2"},{"logon_type":"UserProcess","user":"vagrant","device":"pts/2","pid":2895,"host":"fe80::a00:27ff:fe3a:cda9%enp0s3","timestamp":"2016-03-31T16:45:31.864225+00:00","time_epoch":1459442731,"ip_addr":"fe80:0000:0000:0000:0a00:27ff:fe3a:cda9"},{"logon_type":"UserProcess","user":"vagrant","device":"pts/3","pid":2925,"host":"fe80::a00:27ff:fe00:eec2%enp0s8","timestamp":"2016-03-31T16:47:18.450328+00:00","time_epoch":1459442838,"ip_addr":"fe80:0000:0000:0000:0a00:27ff:fe00:eec2"}]
```

One alternative to JSON, are _JSON Lines_:
```
{"logon_type":"UserProcess","user":"vagrant","device":"pts/0","pid":2776,"host":"10.0.2.2","timestamp":"2016-03-31T16:40:47.513326+00:00","time_epoch":1459442447,"ip_addr":"10.0.2.2"}
{"logon_type":"UserProcess","user":"vagrant","device":"pts/1","pid":2863,"host":"10.0.2.2","timestamp":"2016-03-31T16:44:55.261504+00:00","time_epoch":1459442695,"ip_addr":"10.0.2.2"}
{"logon_type":"UserProcess","user":"vagrant","device":"pts/2","pid":2895,"host":"fe80::a00:27ff:fe3a:cda9%enp0s3","timestamp":"2016-03-31T16:45:31.864225+00:00","time_epoch":1459442731,"ip_addr":"fe80:0000:0000:0000:0a00:27ff:fe3a:cda9"}
{"logon_type":"UserProcess","user":"vagrant","device":"pts/3","pid":2925,"host":"fe80::a00:27ff:fe00:eec2%enp0s8","timestamp":"2016-03-31T16:47:18.450328+00:00","time_epoch":1459442838,"ip_addr":"fe80:0000:0000:0000:0a00:27ff:fe00:eec2"}
```

If you need CSV output, this is how that looks like (this output used the _--csv-header_ option too):
```
LogonType,User,TerminalDevice,PID,Host,Timestamp,TimestampEpoch,RemoteIP
UserProcess,vagrant,pts/0,2776,10.0.2.2,2016-03-31T16:40:47.513326+00:00,1459442447,10.0.2.2
UserProcess,vagrant,pts/1,2863,10.0.2.2,2016-03-31T16:44:55.261504+00:00,1459442695,10.0.2.2
UserProcess,vagrant,pts/2,2895,fe80::a00:27ff:fe3a:cda9%enp0s3,2016-03-31T16:45:31.864225+00:00,1459442731,fe80:0000:0000:0000:0a00:27ff:fe3a:cda9
UserProcess,vagrant,pts/3,2925,fe80::a00:27ff:fe00:eec2%enp0s8,2016-03-31T16:47:18.450328+00:00,1459442838,fe80:0000:0000:0000:0a00:27ff:fe00:eec2
```

Last but not least, if you're a fan of XML, you can also produce an XML document:
```
<?xml version="1.0" encoding="utf-8" ?>
<LogonEntries>
  <LogonEntry type="UserProcess" user="vagrant" terminal="pts/0" pid="2776" host="10.0.2.2" timestamp="2016-03-31T16:40:47.513326+00:00" time_epoch="1459442447" remote_ip="10.0.2.2"></LogonEntry>
  <LogonEntry type="UserProcess" user="vagrant" terminal="pts/1" pid="2863" host="10.0.2.2" timestamp="2016-03-31T16:44:55.261504+00:00" time_epoch="1459442695" remote_ip="10.0.2.2"></LogonEntry>
  <LogonEntry type="UserProcess" user="vagrant" terminal="pts/2" pid="2895" host="fe80::a00:27ff:fe3a:cda9%enp0s3" timestamp="2016-03-31T16:45:31.864225+00:00" time_epoch="1459442731" remote_ip="fe80:0000:0000:0000:0a00:27ff:fe3a:cda9"></LogonEntry>
  <LogonEntry type="UserProcess" user="vagrant" terminal="pts/3" pid="2925" host="fe80::a00:27ff:fe00:eec2%enp0s8" timestamp="2016-03-31T16:47:18.450328+00:00" time_epoch="1459442838" remote_ip="fe80:0000:0000:0000:0a00:27ff:fe00:eec2"></LogonEntry>
</LogonEntries>
```

## License
`mhwho` is licensed under the GNU GPL version 3. Software must be free.
