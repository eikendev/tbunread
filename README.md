<div align="center">
	<h1>tbunread</h1>
	<h4 align="center">
		Unread counts for your <a href="https://www.thunderbird.net/">Thunderbird</a> accounts.
	</h4>
	<p>Watch Thunderbird mail folders and output unread totals in a simple, script-friendly format.</p>
</div>

<p align="center">
	<a href="https://github.com/eikendev/tbunread/actions"><img alt="Build status" src="https://img.shields.io/github/actions/workflow/status/eikendev/tbunread/main.yml?branch=main"/></a>&nbsp;
	<a href="https://crates.io/crates/tbunread"><img alt="License" src="https://img.shields.io/crates/l/tbunread"/></a>&nbsp;
	<a href="https://crates.io/crates/tbunread"><img alt="Version" src="https://img.shields.io/crates/v/tbunread"/></a>&nbsp;
	<a href="https://crates.io/crates/tbunread"><img alt="Downloads" src="https://img.shields.io/crates/d/tbunread"/></a>&nbsp;
</p>

## ✨&nbsp;Why tbunread?

Thunderbird is great, but it does not expose a simple, stable way to read unread counts from the outside. If you want to feed a status bar, a panel widget, or a script, I found no good native way to do that.

tbunread reads Thunderbird's local IMAP index files and prints a compact list of unread totals, ordered exactly the way you want. It updates whenever files change and stays out of the way.

## 🧠&nbsp;How it works

- Configure email accounts by creating symlinks in `ImapMail/tbunread` (see [Configuration](#configuration))
- Setup tbunread to run in the background (see [Recommended setup](#recommended-setup-systemd))
- tbunread watches the default profile for changes
  * Prints counts to stdout and optionally writes them to a file
  * Emits `???` when Thunderbird is not running

## 🚀&nbsp;Installation

Install tbunread using Cargo:

```bash
cargo install tbunread
```

## 📄&nbsp;Usage

### Configuration

tbunread only processes accounts you explicitly link. Create a `tbunread` directory inside your Thunderbird profile's `ImapMail` directory, then add symlinks to each IMAP account directory. (POP3 is not supported.)

Example layout:

```bash
$ pwd
/path/to/.thunderbird/some.profile/ImapMail/tbunread
$ ls -lA
lrwxrwxrwx. (...) 01 -> ../mail.example1.com
lrwxrwxrwx. (...) 02 -> ../mail.example2.com
lrwxrwxrwx. (...) 03 -> ../mail.example3.com
```

The symlink names define the output order (alphabetical).

### Run

Print unread counts to stdout:

```bash
tbunread
```

Write counts to a file:

```bash
tbunread --output /where/you/need/the/output
```

Only write to a file (no stdout):

```bash
tbunread --quiet --output /where/you/need/the/output
```

Adjust the Thunderbird process scan interval (seconds):

```bash
tbunread --interval 10
```

## 🔧&nbsp;Recommended setup (systemd)

For continuous updates, run tbunread as a systemd user service:

```ini
[Unit]
Description=tbunread

[Service]
Type=simple
ExecStart=tbunread --output /where/you/need/the/output
Restart=on-success
RestartSec=5s

[Install]
WantedBy=default.target
```
