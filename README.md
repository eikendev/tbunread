[![Build status](https://img.shields.io/github/workflow/status/eikendev/tbunread/Main)](https://github.com/eikendev/tbunread/actions)
[![License](https://img.shields.io/crates/l/tbunread)](https://crates.io/crates/tbunread)
[![Version](https://img.shields.io/crates/v/tbunread)](https://crates.io/crates/tbunread)
[![Downloads](https://img.shields.io/crates/d/tbunread)](https://crates.io/crates/tbunread)

## About

This script outputs how many emails are unread in each account of Thunderbird.
It will automatically detect your default Thunderbird profile.

## Usage

```
$ tbunread --output /where/you/need/the/output
[*] Reading: /path/to/my/.thunderbird/profiles.ini
[+] Watching: /path/to/my/.thunderbird/some.profile/ImapMail/tbunread
[*] Update: 6 1 1 2
[*] Update: 5 1 1 2
[*] Update: 5 1 2 2
```

To use the script you have to provide it with the email accounts you want to query.
This is done by creating symbolic links in a `tbunread` directory inside the `ImapMail` directory of Thunderbird.
The symbolic links point to one of the IMAP directories (POP3 is not supported).
By naming the links in the alphabetical order of your choice you can also choose the order of the output.

Here is an example of how it might look inside a `tbunread` directory:
```
$ pwd
/path/to/my/.thunderbird/some.profile/ImapMail/tbunread
$ ls -lA
lrwxrwxrwx. (...) 01 -> ../mail.example1.com
lrwxrwxrwx. (...) 02 -> ../mail.example2.com
lrwxrwxrwx. (...) 03 -> ../mail.example3.com
```

## Installation

### From crates.io

```bash
cargo install tbunread
```

## Recommended Setup

I recommend using [systemd](https://systemd.io/) to run the script.
See below for an example on how the service file should look like.

```ini
[Unit]
Description=tbunread

[Service]
Type=fork
ExecStart=tbunread --output /where/you/need/the/output
Restart=on-success
RestartSec=5s

[Install]
WantedBy=default.target
```
