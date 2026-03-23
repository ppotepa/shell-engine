# Shell Quest — Full Feature Inventory

## Cognitos OS / Minix 1.1 Simulation — Complete Feature List

Master reference for everything the simulated Minix should contain.
Organized by category. Each item marked with status:

- 🟢 EXISTS — already implemented
- 🔵 PLANNED — designed, ready to build
- ⚪ PLACEHOLDER — stub/furtka, minimal implementation now, expand later
- 🟡 MYSTERY — spooky/narrative element

---

## I. SYSTEM ARCHITECTURE

### 1. Real Filesystem on Disk

| #   | Feature                                    | Status | Description                                                     |
| --- | ------------------------------------------ | ------ | --------------------------------------------------------------- |
| 1   | Save directory `~/.shell-quest/save/`      | 🔵     | Real folder on host machine. First boot creates it.             |
| 2   | First boot = `formatting /dev/hd1...`      | 🔵     | Winchester task creates the directory tree. Player sees format. |
| 3   | Subsequent boots = `mounting /dev/hd1`     | 🔵     | Existing dir detected, normal mount message.                    |
| 4   | `cat`/`ls`/`cp`/`cd` operate on real files | 🔵     | VFS backed by actual disk I/O.                                  |
| 5   | Disk space tracked per difficulty          | 🔵     | `MachineSpec.DiskTotal/DiskFree`. `cp` fails if full.           |
| 6   | File sizes are real                        | 🔵     | Each file has a byte count. `ls -l` shows sizes.                |
| 7   | `.state.json` persistence                  | 🔵     | Login count, timestamps, quest flags, anomaly state.            |

### 2. Boot Sequence = Real Service Init

| #   | Feature                                  | Status | Description                                         |
| --- | ---------------------------------------- | ------ | --------------------------------------------------- |
| 8   | `clock task` → `ClockService.Init()`     | 🔵     | Starts system timer, uptime counter.                |
| 9   | `memory task` → `MemoryManager.Init()`   | 🔵     | Calculates free RAM from MachineSpec.               |
| 10  | `winchester task` → `DiskManager.Init()` | 🔵     | Opens/creates filesystem. First boot = format.      |
| 11  | `tty task` → `TtyManager.Init()`         | 🔵     | Initializes viewport, allocates 3 virtual consoles. |
| 12  | `ethernet task` → `NetworkStack.Init()`  | 🔵     | Loads `NetworkRegistry`, interface UP.              |
| 13  | `init` → starts `/etc/rc`                | 🟢     | Already in boot sequence.                           |
| 14  | `/etc/rc` → `update`, `cron`, `getty`    | 🟢     | Already emitted as boot text.                       |
| 15  | Each service = `IService` interface      | 🔵     | `Init()`, `Tick(dtMs)`, `Status()`, `Name`.         |

### 3. Time System

| #   | Feature                               | Status | Description                                              |
| --- | ------------------------------------- | ------ | -------------------------------------------------------- |
| 16  | Epoch = `Sep 17 1991 21:00:00 EET`    | 🟢     | Already defined in OperatingSystem.                      |
| 17  | Time ticks 1:1 with real time         | 🔵     | `SimulatedNow = Epoch + total_played_ms`.                |
| 18  | `first_boot` timestamp persisted      | 🔵     | Saved in `.state.json`. Never changes.                   |
| 19  | `last_login` timestamp persisted      | 🔵     | Updated each login. Delta shown on next login.           |
| 20  | `total_session_seconds` accumulated   | 🔵     | Tracks total play time across sessions.                  |
| 21  | `login_count` incremented per session | 🔵     | Drives MOTD variations and narrative escalation.         |
| 22  | `uptime` shows real accumulated time  | 🔵     | `kruuna up 2 days, 3:17` if player returns after 2 days. |
| 23  | `last login: <delta>` on login        | 🔵     | `last login: Wed Sep 17 21:12 (8 hours ago)`             |

### 4. Process Model

| #   | Feature                               | Status | Description                                                                         |
| --- | ------------------------------------- | ------ | ----------------------------------------------------------------------------------- |
| 24  | Process table (`List<Process>`)       | 🟢     | Already exists, drives `ps` and `top`.                                              |
| 25  | Each process has PID, user, CPU%, MEM | 🟢     | Sine-wave animation per tick.                                                       |
| 26  | Shell = process owned by `linus`      | 🟢     | PID 6+, state R (running).                                                          |
| 27  | `ast` has a process (vi) on tty1      | 🔵     | Always present. Uses RAM.                                                           |
| 28  | tty2 unnamed process                  | 🟡     | `(unknown)` in process list. PID exists. CPU 0.0%. Always.                          |
| 29  | `kill <pid>` command                  | ⚪     | Stub: `kill: not permitted` for system processes. Player's shell killable → logout. |
| 30  | Swap simulation on low-RAM difficulty | 🔵     | Processes get `swapped` state when RAM exhausted.                                   |

### 5. Users & Multi-user

| #   | Feature                        | Status | Description                                                                    |
| --- | ------------------------------ | ------ | ------------------------------------------------------------------------------ |
| 31  | `/etc/passwd` with real users  | 🔵     | root, daemon, bin, ast, linus, nobody.                                         |
| 32  | `ast` is a real logged-in user | 🔵     | tty1, session since Sep 15. Has home dir, .plan, mail.                         |
| 33  | `root` exists but inaccessible | 🔵     | `su` always fails. Furtka for future privilege escalation quest.               |
| 34  | tty2 anonymous user            | 🟡     | No name in `who`. Session from `Jan 1 00:00`.                                  |
| 35  | `write <user>` command         | ⚪     | Stub: `write ast` → `ast: not accepting messages`. `write tty2` → no response. |
| 36  | `finger <user>` command        | ⚪     | Shows user info from passwd + .plan. `finger tty2` → `no such user`.           |
| 37  | `id` command                   | ⚪     | `uid=101(linus) gid=10(staff)`                                                 |
| 38  | `groups` command               | ⚪     | `staff operator`                                                               |
| 39  | `passwd` command               | ⚪     | `passwd: only root may change passwords` — furtka.                             |
| 40  | `adduser` command              | ⚪     | `adduser: permission denied` — furtka for future admin quest.                  |

---

## II. SHELL & COMMANDS

### 6. Core Commands (Existing)

| #   | Command          | Status | Notes                         |
| --- | ---------------- | ------ | ----------------------------- |
| 41  | `ls [dir]`       | 🟢     | List directory.               |
| 42  | `cat <file>`     | 🟢     | Display file.                 |
| 43  | `cd <dir>`       | 🟢     | Change directory.             |
| 44  | `pwd`            | 🟢     | Print working directory.      |
| 45  | `cp <src> <dst>` | 🟢     | Copy file. Tracks quest flag. |
| 46  | `ps`             | 🟢     | Process list.                 |
| 47  | `top`            | 🟢     | System monitor.               |
| 48  | `services`       | 🟢     | Service list.                 |
| 49  | `clear`          | 🟢     | Clear screen.                 |
| 50  | `help`           | 🟢     | Command list.                 |
| 51  | `ftp [host]`     | 🟢     | Launch FTP application.       |

### 7. New Environment Commands

| #   | Command       | Status | Output                                                                 |
| --- | ------------- | ------ | ---------------------------------------------------------------------- |
| 52  | `date`        | 🔵     | `Wed Sep 17 21:34:12 EET 1991` (ticking). Date glitch after anomalies. |
| 53  | `uptime`      | 🔵     | Real accumulated uptime + 3 users + load average.                      |
| 54  | `whoami`      | 🔵     | `linus`                                                                |
| 55  | `who`         | 🔵     | 3 users: linus, ast, (unnamed on tty2).                                |
| 56  | `uname [-a]`  | 🔵     | `MINIX 1.1 kruuna 386DX i386 Sep 17 1991`                              |
| 57  | `hostname`    | 🔵     | `kruuna`                                                               |
| 58  | `id`          | ⚪     | `uid=101(linus) gid=10(staff)`                                         |
| 59  | `groups`      | ⚪     | `staff operator`                                                       |
| 60  | `echo <text>` | 🔵     | Prints text. Supports `$USER`, `$HOME`, `$SHELL`.                      |
| 61  | `env`         | ⚪     | Show environment variables (USER, HOME, SHELL, PATH, TERM).            |
| 62  | `export`      | ⚪     | `export: read-only environment` — furtka.                              |
| 63  | `history`     | ⚪     | Shows command history (real, from `.bash_history`).                    |
| 64  | `alias`       | ⚪     | `alias: not supported in sh` — furtka for bash upgrade.                |

### 8. Filesystem Commands

| #   | Command                       | Status | Output                                                               |
| --- | ----------------------------- | ------ | -------------------------------------------------------------------- |
| 65  | `ls -l`                       | 🔵     | Long format: permissions, owner, size, date, name.                   |
| 66  | `ls -a`                       | 🔵     | Show hidden files (.bash_history, .plan, etc).                       |
| 67  | `df`                          | 🔵     | Disk free: `/dev/hd1` and `/dev/hd2` usage from MachineSpec.         |
| 68  | `du [dir]`                    | ⚪     | Disk usage per directory.                                            |
| 69  | `mkdir <dir>`                 | ⚪     | `mkdir: permission denied` outside home. Works in ~/.                |
| 70  | `rm <file>`                   | 🔵     | `rm: permission denied (nice try)` — or works on user-created files. |
| 71  | `mv <src> <dst>`              | ⚪     | Move/rename. Works only in home dir.                                 |
| 72  | `touch <file>`                | ⚪     | Create empty file. Uses disk space.                                  |
| 73  | `chmod`                       | ⚪     | `chmod: operation not permitted` on system files. Furtka.            |
| 74  | `chown`                       | ⚪     | `chown: must be superuser` — furtka.                                 |
| 75  | `find <path> -name <pattern>` | ⚪     | Basic file search. Slow on low-spec machines.                        |
| 76  | `grep <pattern> <file>`       | ⚪     | Search file contents. Essential UNIX tool.                           |
| 77  | `wc <file>`                   | ⚪     | Word/line/byte count.                                                |
| 78  | `head`/`tail`                 | ⚪     | First/last N lines.                                                  |
| 79  | `file <path>`                 | ⚪     | `file: linux-0.01.tar.Z: compressed data` / `ASCII text` etc.        |

### 9. Network Commands

| #   | Command                | Status | Output                                                                           |
| --- | ---------------------- | ------ | -------------------------------------------------------------------------------- |
| 80  | `ping <host>`          | 🔵     | Full ping with latency, jitter, stats. Anomalies.                                |
| 81  | `netstat`              | ⚪     | Listening ports (ftp 21, telnet 23, http 80), packet counters.                   |
| 82  | `ifconfig`             | ⚪     | `ne2000: 130.xxx.xxx.xxx netmask 255.255.255.0 UP`.                              |
| 83  | `nslookup <host>`      | ⚪     | DNS lookup via NetworkRegistry. Anomaly hosts → weird responses.                 |
| 84  | `traceroute <host>`    | ⚪     | Shows hop path. Normal hosts: 5-12 hops. Anomalies: hops disappear/repeat.       |
| 85  | `telnet <host> <port>` | ⚪     | Stub: `telnet: connection refused` or banner for known hosts. Furtka for future. |
| 86  | `wget`/`fetch`         | ⚪     | `wget: command not found` — didn't exist yet. Furtka.                            |
| 87  | `mail`                 | 🔵     | See section III — Applications.                                                  |

### 10. System Commands

| #   | Command                    | Status | Output                                                                                 |
| --- | -------------------------- | ------ | -------------------------------------------------------------------------------------- |
| 88  | `dmesg`                    | 🔵     | Kernel ring buffer. Spooky additions after anomalies.                                  |
| 89  | `mount`                    | ⚪     | Shows mounted filesystems (`/dev/hd1` on `/`, `/dev/hd2` on `/usr`).                   |
| 90  | `free`                     | ⚪     | Memory stats from MachineSpec: total, used, free, swap.                                |
| 91  | `kill <pid>`               | ⚪     | `kill: not permitted` for system. Kills own processes.                                 |
| 92  | `nice`/`renice`            | ⚪     | `nice: permission denied` — furtka.                                                    |
| 93  | `crontab`                  | ⚪     | `crontab: no changes made` — view-only.                                                |
| 94  | `at`                       | ⚪     | `at: command scheduling disabled` — furtka.                                            |
| 95  | `sync`                     | ⚪     | `(syncing disks...)` — writes state to disk immediately.                               |
| 96  | `shutdown`/`halt`/`reboot` | 🔵     | `must be superuser` — furtka for root access quest.                                    |
| 97  | `init`                     | ⚪     | `init: must be run as PID 1`                                                           |
| 98  | `fsck`                     | ⚪     | `fsck: /dev/hd1: clean` — or after anomalies: `fsck: /dev/hd1: UNEXPECTED INODE COUNT` |

### 11. Text Tools (Minix 1.1 era-accurate)

| #   | Command                 | Status | Output                                                                    |
| --- | ----------------------- | ------ | ------------------------------------------------------------------------- |
| 99  | `more <file>`           | ⚪     | Paged file viewer. `--More--` prompt.                                     |
| 100 | `sort <file>`           | ⚪     | Sort lines alphabetically.                                                |
| 101 | `uniq`                  | ⚪     | Remove duplicate lines.                                                   |
| 102 | `tee`                   | ⚪     | `tee: not enough file descriptors` on low difficulty.                     |
| 103 | `tr`                    | ⚪     | Translate characters. Basic UNIX text pipeline tool.                      |
| 104 | `sed`                   | ⚪     | `sed: not installed` — Minix 1.1 didn't ship sed. Furtka.                 |
| 105 | `awk`                   | ⚪     | `awk: not installed` — same. Furtka for future tool unlock.               |
| 106 | `diff`                  | ⚪     | Compare files. Could be useful for quest verification.                    |
| 107 | `compress`/`uncompress` | ⚪     | `.Z` file handling. `uncompress linux-0.01.tar.Z` → works (creates .tar). |
| 108 | `tar`                   | ⚪     | `tar xf linux-0.01.tar` → extracts contents. Furtka for exploration.      |
| 109 | `od` (octal dump)       | ⚪     | `od -c <file>` → hex dump. For the hardcore.                              |

### 12. Man Pages

| #   | Command             | Status | Topics                                                    |
| --- | ------------------- | ------ | --------------------------------------------------------- |
| 110 | `man <topic>`       | 🔵     | ftp, ls, cat, cp, chmod, ping. Others: `No manual entry`. |
| 111 | `man man`           | ⚪     | Meta-manpage explaining the manual system.                |
| 112 | `man hier`          | ⚪     | Filesystem hierarchy explained. Educational!              |
| 113 | `man 5 passwd`      | ⚪     | `/etc/passwd` format explanation.                         |
| 114 | `apropos <keyword>` | ⚪     | `apropos: whatis database not built` — furtka.            |

---

## III. APPLICATIONS (IApplication)

### 13. Shell (`ShellApplication`) 🟢

Already exists. The base — always at bottom of stack.

### 14. FTP Client (`FtpApplication`) 🟢

Already exists. Quest-critical. `open`, `binary`, `put`.

### 15. Mail Reader (`MailApplication`) 🔵

**Class:** `MailApplication : IApplication`

Launched via `mail` command. Interactive mailbox:

```
Mail version 5.0.  Type ? for help.
"/var/spool/mail/linus": 3 messages 2 new

 N  1  op@kruuna          Tue Sep 16  Welcome
 N  2  ast@cs.vu.nl       Tue Sep 16  re: uploading to funet
    3  cron-daemon         Wed Sep 17  Cron output

&
```

Commands: `1`-`N` (read message), `d <n>` (delete), `r <n>` (reply stub), `q` (quit), `?` (help).

New mail appears dynamically:

- Session 1: welcome + ast hint
- Session 2: student gossip ("anyone get weird ping errors?")
- Session 3: operator warning ("report anomalies")
- After all anomalies: unsigned message with no From header, body: `you're looking in the right places.`

### 16. Text Editor (`EdApplication`) ⚪

**Class:** `EdApplication : IApplication`

`ed` — the standard UNIX line editor. Minix 1.1 shipped `ed`, not `vi`.

```
$ ed notes/todo.txt
47
:
```

Minimal: `a` (append), `w` (write), `q` (quit), `p` (print), `d` (delete line).
Allows player to create and edit files. Opens doors for puzzle mechanics.

`vi` easter egg: "insufficient memory" → hints player to use `ed` instead.

### 17. Telnet Client (`TelnetApplication`) ⚪

**Class:** `TelnetApplication : IApplication`

```
$ telnet cs.vu.nl
Trying 130.37.24.3...
Connected to cs.vu.nl.
Escape character is '^]'.

SunOS 4.1.1 (cs) (ttyp3)

login:
```

Stub: shows banner, accepts login attempt, always fails with `Login incorrect.`
Furtka for future quest where player actually connects somewhere.

For anomaly hosts: `telnet google.com` →

```
Trying ...
net: address resolved to unallocated block
telnet: Unable to connect to remote host: Network is unreachable
(but connection was briefly acknowledged)
```

### 18. Talk (`TalkApplication`) ⚪

**Class:** `TalkApplication : IApplication`

`talk ast` → `[Waiting for ast to respond...]` → never responds.
`talk tty2` → `[Connection established]` → empty screen → after 5 seconds:

```
[Connection lost]
talk: remote party disconnected
```

Something was listening. Something disconnected. Furtka.

---

## IV. SERVICES (IService)

### 19. Service Architecture

Each service implements:

```csharp
interface IService
{
    string Name { get; }
    ServiceState State { get; }  // Running, Stopped, Degraded, Unknown
    void Init(IOperatingSystem os);
    void Tick(ulong dtMs);
    string StatusLine();
}
```

### 20. Service List

| #   | Service   | Status | Description                                                            |
| --- | --------- | ------ | ---------------------------------------------------------------------- |
| 20a | `telnetd` | 🟢     | Listening on port 23. Status: Running.                                 |
| 20b | `httpd`   | 🟢     | Listening on port 80. Status: Running.                                 |
| 20c | `ftpd`    | 🟢     | Listening on port 21. Status: Running.                                 |
| 20d | `netd`    | 🟢     | Network daemon. Status: Running.                                       |
| 20e | `maild`   | 🟢     | Mail delivery. Status: Running.                                        |
| 20f | `cron`    | 🟢     | Task scheduler. Status: Running.                                       |
| 20g | `update`  | 🔵     | Disk sync daemon. Runs `sync` every 30s.                               |
| 20h | `inetd`   | ⚪     | Internet super-daemon. Manages telnetd/httpd/ftpd.                     |
| 20i | `lpd`     | ⚪     | Printer daemon. Status: `Stopped (no printer)`.                        |
| 20j | `syslogd` | ⚪     | System logger. Writes to `/var/log/messages`.                          |
| 20k | `named`   | ⚪     | DNS resolver cache. After anomalies: `Degraded (cache inconsistency)`. |

---

## V. NETWORK (IExternalServer)

### 21. Server Registry

| #   | Host            | IP              | Type     | Services                |
| --- | --------------- | --------------- | -------- | ----------------------- |
| 21a | `nic.funet.fi`  | `128.214.6.100` | Normal   | FTP (quest target)      |
| 21b | `ftp.funet.fi`  | `128.214.6.100` | Normal   | FTP (alias)             |
| 21c | `cs.vu.nl`      | `130.37.24.3`   | Normal   | Telnet banner, FTP      |
| 21d | `sun.com`       | `192.9.9.1`     | Normal   | Ping-only (no FTP)      |
| 21e | `helsinki.fi`   | `128.214.1.1`   | Normal   | Ping-only               |
| 21f | `ftp.uu.net`    | `192.48.96.9`   | Normal   | FTP (UUNET archives)    |
| 21g | `mit.edu`       | `18.72.2.1`     | Normal   | Ping-only, high latency |
| 21h | `uunet.uu.net`  | `192.48.96.2`   | Normal   | Ping-only               |
| 21i | `localhost`     | `127.0.0.1`     | Loopback | —                       |
| 21j | `kruuna`        | `127.0.0.1`     | Loopback | —                       |
| 21k | `google.com`    | —               | Anomaly  | Temporal anomaly        |
| 21l | `github.com`    | —               | Anomaly  | Temporal anomaly        |
| 21m | `wikipedia.org` | —               | Anomaly  | Temporal anomaly        |

### 22. FTP Servers (connectable)

| #   | Host           | Remote Dir      | Files Available                        |
| --- | -------------- | --------------- | -------------------------------------- |
| 22a | `nic.funet.fi` | `/pub/OS/Linux` | `README`, upload target                |
| 22b | `ftp.uu.net`   | `/pub/`         | Index of UUNET archives (stub listing) |
| 22c | `cs.vu.nl`     | `/pub/minix/`   | Minix source tarballs (stub listing)   |

Player can `ftp ftp.uu.net` and browse — flavor content, no quest impact.

---

## VI. FILESYSTEM CONTENTS

### 23. System Files

| #   | Path               | Content                                                       |
| --- | ------------------ | ------------------------------------------------------------- |
| 23a | `/etc/passwd`      | root, daemon, bin, ast, linus, nobody                         |
| 23b | `/etc/motd`        | Rotating message pool (see extras.impl.md A1)                 |
| 23c | `/etc/rc`          | Startup script listing (update, cron, getty)                  |
| 23d | `/etc/hostname`    | `kruuna`                                                      |
| 23e | `/etc/hosts`       | Static host table (localhost, kruuna, nic.funet.fi, cs.vu.nl) |
| 23f | `/etc/services`    | Port mappings (ftp 21, telnet 23, smtp 25, http 80)           |
| 23g | `/etc/resolv.conf` | `nameserver 128.214.1.1`                                      |
| 23h | `/etc/profile`     | Shell startup: sets PATH, TERM, exports                       |

### 24. Home Directory (`/home/linus/`)

| #   | Path                          | Content                                                                          |
| --- | ----------------------------- | -------------------------------------------------------------------------------- |
| 24a | `linux-0.01/RELNOTES-0.01`    | 🟢 Authentic Linus release notes                                                 |
| 24b | `linux-0.01/linux-0.01.tar.Z` | 🟢 Archive placeholder (73091 bytes)                                             |
| 24c | `linux-0.01/README`           | 🟢 Upload instructions (hint)                                                    |
| 24d | `linux-0.01/bash.Z`           | 🟢 Compressed binary placeholder                                                 |
| 24e | `linux-0.01/update.Z`         | 🟢 Compressed daemon placeholder                                                 |
| 24f | `mail/welcome.txt`            | 🟢 Welcome mail from operator                                                    |
| 24g | `mail/ast.txt`                | 🔵 Hint mail from Tanenbaum                                                      |
| 24h | `notes/starter.txt`           | 🟢 Getting started notes                                                         |
| 24i | `.bash_history`               | 🔵 Previous failed session (ascii mode hint)                                     |
| 24j | `.profile`                    | ⚪ User shell config                                                             |
| 24k | `.plan`                       | ⚪ Linus's .plan: "working on something. will post to comp.os.minix when ready." |

### 25. System Directories

| #   | Path                           | Content                                                                        |
| --- | ------------------------------ | ------------------------------------------------------------------------------ |
| 25a | `/usr/ast/README`              | 🔵 Tanenbaum's "MINIX is for teaching" message                                 |
| 25b | `/usr/ast/.plan`               | 🔵 Working on MINIX 2.0, teaching next semester                                |
| 25c | `/usr/ast/minix-2.0-notes.txt` | ⚪ Partial design notes for next MINIX version                                 |
| 25d | `/usr/bin/`                    | ⚪ System binaries listing (ls, cat, cp, etc.)                                 |
| 25e | `/usr/man/`                    | ⚪ Man page directory                                                          |
| 25f | `/usr/src/`                    | ⚪ `ls`: `minix/` — Tanenbaum's source tree. Not readable (permission denied). |
| 25g | `/usr/lib/`                    | ⚪ Libraries listing                                                           |
| 25h | `/bin/`                        | ⚪ Core binaries (sh, echo, test)                                              |
| 25i | `/dev/null`                    | 🔵 Empty on read.                                                              |
| 25j | `/dev/random`                  | 🔵 Garbled output.                                                             |
| 25k | `/dev/console`                 | ⚪ `[device — cannot read directly]`                                           |
| 25l | `/proc/version`                | 🟡 Minix version. Glitches after anomalies ("Li" append).                      |

### 26. Log Files

| #   | Path                    | Content                                                       |
| --- | ----------------------- | ------------------------------------------------------------- |
| 26a | `/var/log/messages`     | 🔵 System log. Includes tty2 anonymous login.                 |
| 26b | `/var/log/net.trace`    | 🟡 Appears after anomaly pings. Grows per anomaly.            |
| 26c | `/var/log/cron.log`     | ⚪ Cron execution log. After anomalies: clock drift warnings. |
| 26d | `/var/log/auth.log`     | ⚪ Login/logout records. Shows (unknown) on tty2.             |
| 26e | `/var/spool/mail/linus` | 🔵 Mailbox file (used by `mail` application).                 |

### 27. Temp Files (flavor)

| #   | Path                            | Content                                                      |
| --- | ------------------------------- | ------------------------------------------------------------ |
| 27a | `/tmp/thesis-FINAL-v3-REAL.bak` | `[binary file — cannot display]`                             |
| 27b | `/tmp/core`                     | `[core dump — process 'cc1' signal 11 (segmentation fault)]` |
| 27c | `/tmp/nroff-err.log`            | nroff warnings about fonts                                   |
| 27d | `/tmp/.Xauthority`              | `[binary file]` — X on Minix?                                |
| 27e | `/tmp/.lock-ast`                | Empty lockfile. ast left a session running.                  |

---

## VII. EASTER EGGS & MYSTERIES

### 28. Command Responses (IEasterEgg)

| #   | Input                      | Response                                                   | Type         |
| --- | -------------------------- | ---------------------------------------------------------- | ------------ |
| 28a | `emacs`                    | `emacs: not installed. only vi available on this system.`  | Humor        |
| 28b | `vi`                       | _(200ms delay)_ `vi: insufficient memory`                  | Humor        |
| 28c | `rm` (anything)            | `rm: permission denied (nice try)`                         | Safety       |
| 28d | `su`                       | `su: incorrect password`                                   | Furtka       |
| 28e | `su` (difficulty=SU)       | `su: you chose this name, didn't you?`                     | Meta         |
| 28f | `shutdown`/`halt`/`reboot` | `must be superuser`                                        | Furtka       |
| 28g | `make`                     | `make: no targets. nothing to do.`                         | Humor        |
| 28h | `gcc`                      | `gcc: not installed. try Amsterdam Compiler Kit.`          | Era-accurate |
| 28i | `cc`                       | `cc: Amsterdam Compiler Kit 3.0` then `cc: no input files` | Era-accurate |
| 28j | `exit`                     | `logout` → back to login                                   | Functional   |
| 28k | `finger ast`               | Shows ast's .plan file                                     | Immersion    |
| 28l | `finger linus`             | Login name, real name, home dir                            | Immersion    |
| 28m | `finger tty2`              | `finger: tty2: no such user.`                              | Mystery      |
| 28n | `echo hello`               | `hello`                                                    | Functional   |
| 28o | `cat /dev/null`            | _(no output)_                                              | Correct      |
| 28p | `cat /dev/random`          | Garbled characters                                         | Humor        |
| 28q | `passwd`                   | `passwd: only root may change passwords`                   | Furtka       |
| 28r | `adduser`                  | `adduser: permission denied`                               | Furtka       |

### 29. Stateful Easter Eggs

| #   | Input                   | Behavior                                                           | Type     |
| --- | ----------------------- | ------------------------------------------------------------------ | -------- |
| 29a | `minix` (1st, 2nd time) | _(silence — no output, no error)_                                  | Spooky   |
| 29b | `minix` (3rd time)      | `minix: I know.` — then never again                                | Spooky   |
| 29c | `linux`                 | `linux: command not found (not yet)`                               | Humor    |
| 29d | `linux --help`          | Full quest walkthrough (5 steps)                                   | Lifeline |
| 29e | `hello`                 | `hello: unknown command` (1st). After 10 commands: `hello, linus.` | Spooky   |

### 30. Anomaly-Triggered Events

These only appear AFTER the player has pinged anomaly hosts:

| #   | Trigger             | Event                                                      | After N anomalies |
| --- | ------------------- | ---------------------------------------------------------- | ----------------- |
| 30a | `date`              | ~5% chance: flashes future date (2026), corrects to 1991   | 3                 |
| 30b | `cat /proc/version` | ~5% chance: appends `Li` (truncated)                       | 2                 |
| 30c | `dmesg`             | Final line: `[????] process 0: unnamed: started`           | 3                 |
| 30d | `cron.log`          | `cron: /usr/lib/atrun (skipped — clock drift detected)`    | 1                 |
| 30e | `fsck`              | `fsck: /dev/hd1: UNEXPECTED INODE COUNT`                   | 2                 |
| 30f | `services`          | `named` status changes to `Degraded (cache inconsistency)` | 1                 |
| 30g | `netstat`           | Extra line: `???  0.0.0.0:??     *:*  UNKNOWN`             | 3                 |
| 30h | `/var/log/messages` | New entry: `kernel: routing anomaly on eth0`               | 2                 |
| 30i | `top`               | tty2 process briefly shows CPU spike (0.0% → 12.4% → 0.0%) | 3                 |
| 30j | MOTD                | `kernel: routing table integrity check... [WARN]`          | 3                 |

### 31. Post-Quest Mystery

After successfully uploading `linux-0.01.tar.Z` in binary mode:

| #   | Event                   | Description                                                    |
| --- | ----------------------- | -------------------------------------------------------------- |
| 31a | New file appears        | `~/note.txt`: `well done. more to come. — ?`                   |
| 31b | tty2 logout             | `who` shows only 2 users now. tty2 is gone.                    |
| 31c | `/var/log/messages`     | `login: session closed for (unknown) on tty2`                  |
| 31d | Final MOTD              | `This machine's story isn't over. But yours here is, for now.` |
| 31e | `minix` command changes | `minix: thank you.` — once, then silence forever               |

---

## VIII. NARRATIVE THREADS

### 32. Thread: The Upload (Main Quest)

```
Player discovers linux-0.01/ → reads README → learns about funet →
uses ftp → (may fail with ascii) → discovers binary mode →
uploads successfully → quest complete
```

Hint sources: `.bash_history` → `mail/ast.txt` → `README` → `man ftp` → `linux --help`

### 33. Thread: The Anonymous User (Mystery)

```
`who` shows 3 users → tty2 has no name → `finger` fails →
`/var/log/messages` shows "(unknown)" login →
`write tty2` gets no response → `talk tty2` briefly connects then drops →
After quest complete: tty2 disappears → log shows session closed
```

Never explained. Was it the system watching? A future echo? A ghost in the machine?

### 34. Thread: Tanenbaum (Mentor Arc)

```
ast on tty1 → his .plan (working on MINIX 2.0) → his README →
his mail (binary mode hint) → his process (always editing fs.c) →
his minix-2.0-notes.txt → foreshadows the debate that hasn't happened yet
```

The player is Linus. ast is his professor. The dynamic is already in tension —
Linus is building an OS behind his mentor's back, on his mentor's machine.

### 35. Thread: Temporal Anomalies (Deep Lore)

```
ping google.com → weird error → /var/log/net.trace →
ping github.com → weirder error → trace grows →
ping wikipedia.org → weirdest error → trace complete →
System starts glitching: date flashes, version corrupts, cron drifts →
After quest: anomalies stop. tty2 leaves. note.txt appears.
```

The implication: something from the future was watching this moment.
The moment Linux was uploaded. The moment everything changed.
And now that it's done — it's gone.

### 36. Thread: Privilege Escalation (Future Quest Furtka)

Many commands hint at root access:
`su`, `shutdown`, `passwd`, `adduser`, `chmod`, `chown`, `kill (system)`,
`nice`, `/usr/src/` (permission denied)

These are all doors that say "not yet." Future quests could unlock:

- Getting root via a kernel exploit
- Compiling the Amsterdam Compiler Kit
- Building a custom program
- Gaining access to `/usr/src/minix/`

### 37. Thread: The Network Expands (Future Quest Furtka)

Telnet stubs, multiple FTP servers, `wget` not existing yet.
Future quests could involve:

- Connecting to cs.vu.nl to read Tanenbaum's files
- Downloading patches from ftp.uu.net
- Building a web browser (1993 — Mosaic era)
- The Internet grows: new hosts appear in DNS over time

---

## IX. PROGRESSION MODEL

### 38. Session-Based Escalation

| Login #    | What changes                                                |
| ---------- | ----------------------------------------------------------- |
| 1          | Fresh boot. 2 mail messages. Standard MOTD.                 |
| 2          | "Welcome back." MOTD variant. Uptime reflects real absence. |
| 3          | New mail from student. MOTD mentions network check.         |
| 4          | cron.log grows. System feels lived-in.                      |
| 5+         | If anomalies discovered: system starts degrading subtly.    |
| Post-quest | Resolution. Calm. tty2 gone. Final note.                    |

### 39. Command Unlocks (Future)

| Quest Stage    | New Commands Available                               |
| -------------- | ---------------------------------------------------- |
| Prologue       | ls, cat, cd, cp, ftp, ping, mail, man, ps, top, etc. |
| After upload   | tar, uncompress (can now explore linux source)       |
| Root access    | su (works), kill, chmod, shutdown, mount             |
| Compiler quest | cc (works), make (works), ld                         |
| Network quest  | telnet (connects), wget (appears), finger (remote)   |

---

## X. TECHNICAL IMPLEMENTATION NOTES

### 40. Save Format

```
~/.shell-quest/save/
├── fs/                    ← real filesystem tree
│   ├── etc/
│   ├── home/linus/
│   ├── tmp/
│   ├── usr/
│   └── var/
└── .state.json            ← meta state
```

`.state.json`:

```json
{
  "version": 1,
  "difficulty": "I CAN EXIT VIM",
  "first_boot_utc": "2026-03-22T01:55:00Z",
  "last_login_utc": "2026-03-22T09:30:00Z",
  "total_played_ms": 347000,
  "login_count": 3,
  "epoch_start": "1991-09-17T21:00:00",
  "quest": {
    "ftp_transfer_mode": "binary",
    "upload_attempted": true,
    "upload_success": true,
    "backup_made": false,
    "anomalies_discovered": ["google.com", "github.com"],
    "anomaly_count": 2,
    "minix_command_count": 1,
    "tty2_investigated": true,
    "quest_complete": false
  },
  "mail": {
    "read": ["welcome.txt"],
    "unread": ["ast.txt", "student-01.txt"]
  },
  "bash_history": [
    "ls",
    "cat notes/starter.txt",
    "cd linux-0.01",
    "ftp nic.funet.fi"
  ]
}
```

### 41. Resource Accounting

| Resource         | Source                     | Tracked By                                               |
| ---------------- | -------------------------- | -------------------------------------------------------- |
| RAM              | `MachineSpec.RamKb`        | `MemoryManager` — processes consume, swap if exceeded    |
| Disk             | `MachineSpec.DiskFreeKb`   | `DiskManager` — file ops deduct, `df` reports real usage |
| CPU              | Per-process sine wave      | `top`/`ps` — visual only, no real throttling             |
| Network          | `MachineSpec.NicSpeedKbps` | Ping latency scaling, FTP transfer speed                 |
| File descriptors | `MachineSpec.MaxOpenFiles` | Limit on `tee`, `find`, concurrent operations            |
| Max processes    | `MachineSpec.MaxProcesses` | Can't fork if at limit (future)                          |

# Shell Quest — Extras Implementation Guide

Prologue scene (Minix 1.1, September 1991) — immersion, hints, easter eggs, spooky undertones.

All features live in `cognitos-os` sidecar (C#) unless noted otherwise.

**Design philosophy:** Everything looks realistic. A real Minix system from 1991.
But something is slightly off. The machine knows things it shouldn't.
Not horror — just a quiet wrongness that rewards the curious.

---

## 0) Architecture Conventions

### 0A · Every interactive program = `IApplication`

Each program the player can "enter" (shell, ftp, future: mail client, editor)
must be a C# class implementing `IApplication`:

```csharp
interface IApplication
{
    string PromptPrefix(UserSession session);
    void OnEnter(UserSession session);
    void OnExit(UserSession session);
    ApplicationResult HandleInput(string input, UserSession session);
}
```

Pushed/popped via `ApplicationStack`. Input always routes to topmost app.

### 0B · Every shell command = `ICommand`

One-shot commands (ls, cat, ping, fortune, etc.) implement:

```csharp
interface ICommand
{
    string Name { get; }
    IReadOnlyList<string> Aliases { get; }
    CommandResult Execute(CommandContext ctx);
}
```

Registered in `Program.cs` → `MinixOperatingSystem` → `CommandIndex` dictionary.

### 0C · `IExternalServer` — network host abstraction

New interface for any reachable network entity:

```csharp
interface IExternalServer
{
    string Hostname { get; }
    IReadOnlyList<string> Aliases { get; }  // DNS aliases
    string IpAddress { get; }
    int BasePingMs { get; }
    ServerType Type { get; }  // Ftp, Http, Ping-only, Anomaly
}
```

All known hosts registered in a `NetworkRegistry`:

```csharp
sealed class NetworkRegistry
{
    Dictionary<string, IExternalServer> _hosts;  // hostname → server

    IExternalServer? Resolve(string hostname);
    PingResult Ping(string hostname, MachineSpec spec);
}
```

This replaces the current static `DnsTable` in `FtpApplication`.
FTP, ping, and future network commands all resolve through `NetworkRegistry`.

**Server types:**

| Type          | Behavior                                         |
| ------------- | ------------------------------------------------ |
| `Normal`      | Reachable, responds to ping, may host FTP        |
| `PingOnly`    | Responds to ping but no services                 |
| `Anomaly`     | Temporal anomaly — spooky error sequence         |
| `Unreachable` | Known IP but times out (simulates 1991 downtime) |

### 0D · `IEasterEgg` — fallback command responses

For commands that aren't real programs but deserve a response:

```csharp
interface IEasterEgg
{
    string Trigger { get; }
    EasterEggResult Handle(string fullInput, CommandContext ctx);
}
```

Checked after `CommandIndex` lookup fails, before "command not found".
Allows stateful eggs (e.g., `minix` counting how many times called).

---

## A) Environment / Immersion Commands

### A1 · `/etc/motd` — Message of the Day

Display after login, before shell prompt. Random from pool:

```
System maintenance Sunday 22:00-06:00. Plan accordingly.
```

```
New: nic.funet.fi mirrors now available. See /pub for index.
```

```
Please do not leave sessions idle. Other students need access.
```

```
Reminder: back up your files regularly.
```

```
Notice: irregular network activity logged. Probably nothing. -op
```

Last one is subtle — foreshadows the anomalies. Only appears ~20% of the time.

**Implementation:** `AppHost.cs` — after login success, before shell push.
Add `MotdPool` as `string[]` in `AppHost`, pick via `Random.Shared.Next()`.

---

### A2 · `fortune` command

**Class:** `FortuneCommand : ICommand`

Random quote per invocation:

```
"Real programmers don't use Pascal." — Unknown
```

```
"The number of bugs in any program is at least one more." — Lubarsky
```

```
RFC 1149: A Standard for the Transmission of IP Datagrams on Avian Carriers.
```

```
"I'd rather write programs to write programs than write programs." — Dick Sites
```

```
"Unix is user-friendly. It's just picky about who its friends are." — Anonymous
```

```
"xK#9fZ!m@2vL&w*0...Q" — /dev/random
```

**Spooky variant** (~10% chance, late-game only):

```
"The best programs are the ones that haven't been written yet." — ????
```

The attribution is literally four question marks. No context. No explanation.

---

### A3 · `uptime` command

**Class:** `UptimeCommand : ICommand`

```
 21:34:12 up 127 days,  4:33,  3 users,  load average: 0.42, 0.38, 0.31
```

Uptime ticks from `MachineState.UptimeMs` (already tracked).
User count is always 3. Who are the other two? Nobody knows.

If the player runs `who`:

```
linus    tty0     Sep 17 21:12
ast      tty1     Sep 15 09:41
         tty2     Jan  1 00:00
```

Third user: **no name, logged in January 1st at midnight.** No further explanation.

---

### A4 · `date` command

**Class:** `DateCommand : ICommand`

```
Wed Sep 17 21:34:12 EET 1991
```

Ticks from boot epoch. Perfectly normal.

**Except:** if the player has pinged all 3 anomaly hosts, running `date`
once more has a ~5% chance of briefly showing:

```
Wed Sep 17 21:34:12 EET 1991
```

...then the line clears and reprints:

```
Thu Mar 22 01:42:00 EET 2026
```

...then immediately corrects to 1991 again. Like a glitch. One frame.
(Implementation: emit the future date, then immediately emit corrected date
with a 50ms delay. The player sees it flash.)

---

### A5 · `uname` command

**Class:** `UnameCommand : ICommand`

`uname` → `MINIX`
`uname -a` → `MINIX 1.1 kruuna 386DX i386 Sep 17 1991`
`uname -r` → `1.1`

Perfectly normal. No tricks here. This is the straight man.

---

### A6 · `whoami` command

**Class:** `WhoamiCommand : ICommand`

`whoami` → `linus`

No tricks. Identity anchor.

---

### A7 · `who` command

**Class:** `WhoCommand : ICommand`

```
linus    tty0     Sep 17 21:12
ast      tty1     Sep 15 09:41
         tty2     Jan  1 00:00
```

See A3. The empty-name user is the mystery. `finger tty2` could return nothing
or "no such user" — the system has no record of who's on tty2.

---

### A8 · Disk seek flavor

When `cat` reads a file > 3 lines, prepend:

```
hd1: read sectors 1104-1247...
```

Sector numbers derived from filename hash. Short delay (50-200ms) scaled by difficulty.

**Where:** `CatCommand.Execute()` — before output lines.

---

## B) Ping — Network Probing

### B1 · Architecture

**New class:** `PingCommand : ICommand`

Resolves host via `NetworkRegistry`. Behavior depends on `ServerType`.

Latency jitter: base ±15%, scaled by NIC speed from `MachineSpec`.

### B2 · Known hosts (1991-era, reachable)

Implement as `NormalServer : IExternalServer`:

| Host           | IP              | Base Latency | Note                   |
| -------------- | --------------- | ------------ | ---------------------- |
| `nic.funet.fi` | `128.214.6.100` | 47ms         | Quest target           |
| `ftp.funet.fi` | `128.214.6.100` | 47ms         | Alias → same server    |
| `cs.vu.nl`     | `130.37.24.3`   | 112ms        | Tanenbaum's university |
| `sun.com`      | `192.9.9.1`     | 203ms        | Sun Microsystems       |
| `helsinki.fi`  | `128.214.1.1`   | 12ms         | Local university       |
| `ftp.uu.net`   | `192.48.96.9`   | 189ms        | UUNET archives         |
| `localhost`    | `127.0.0.1`     | 0.01ms       | Loopback               |
| `kruuna`       | `127.0.0.1`     | 0.01ms       | Self                   |

Output (3 echo lines + stats):

```
PING nic.funet.fi (128.214.6.100): 56 data bytes
64 bytes from 128.214.6.100: icmp_seq=0 ttl=52 time=47ms
64 bytes from 128.214.6.100: icmp_seq=1 ttl=52 time=51ms
64 bytes from 128.214.6.100: icmp_seq=2 ttl=52 time=44ms
--- nic.funet.fi ping statistics ---
3 packets transmitted, 3 received, 0% packet loss
round-trip min/avg/max = 44/47/51 ms
```

### B3 · Unknown hosts

```
ping: unknown host anything.xyz
```

One line. Standard Minix behavior.

### B4 · The Three Anomalies

Implement as `AnomalyServer : IExternalServer`:

These hosts do not exist in 1991. But the network... almost finds them.

**google.com:**

```
PING google.com ... resolving
net: forward lookup failed
net: retrying via alternate root
... no route to host
ping: transmit failed (unreachable) [0xFE]
note: unexpected partial route trace logged to /var/log/net.trace
```

**github.com:**

```
PING github.com ... resolving
net: name resolution returned inconclusive
net: authority record points to unallocated block
... request timed out
ping: host not found, but 3 hops responded (unexpected)
note: see /var/log/net.trace
```

**wikipedia.org:**

```
PING wikipedia.org ... resolving
net: forward lookup: NXDOMAIN
net: anomaly: received partial ICMP echo from unregistered AS
... connection interrupted
ping: unknown network error [0xFF]
note: logged to /var/log/net.trace
```

Each anomaly takes 2-3 seconds with line-by-line output (simulated timeout).
The error messages are different each time — they're not failing the same way.

**Key detail:** these are NOT `unknown host`. The system _tries_ to resolve them
and _partially succeeds_. That's the unsettling part.

### B5 · `/var/log/net.trace`

File in VFS. Only appears after first anomaly ping. Grows with each:

After 1 anomaly:

```
[warn] unresolvable host returned partial route data
[warn] destination network not yet allocated by IANA
```

After 2:

```
[warn] 2 unresolvable hosts returned partial route data
[warn] destination networks not yet allocated by IANA
[warn] route fragments suggest future allocation
```

After all 3:

```
[warn] 3 unresolvable hosts returned partial route data
[warn] destination networks not yet allocated by IANA
[warn] temporal inconsistency detected in routing tables
[    ] ...this shouldn't happen.
```

The last line has no log level. Just empty brackets and a message that
breaks the fourth wall just enough.

**Implementation:** Track anomaly count in `QuestState`. Generate file
content dynamically in VFS based on count.

---

## C) Filesystem Extras

All new files added to `ZipVirtualFileSystem` seed data.

### C1 · `/etc/passwd`

```
root:x:0:0:Charlie Root:/root:/bin/sh
daemon:x:1:1:System Daemon:/usr/sbin:/bin/false
bin:x:2:2:Binary:/bin:/bin/false
ast:x:100:10:Andy S. Tanenbaum:/usr/ast:/bin/sh
linus:x:101:10:Linus B. Torvalds:/home/linus:/bin/sh
nobody:x:65534:65534:Nobody:/nonexistent:/bin/false
```

Standard. No surprises. Grounds the player in realism.

### C2 · `/tmp` contents

`ls /tmp`:

```
thesis-FINAL-v3-REAL.bak    core    nroff-err.log    .Xauthority
```

`cat thesis-FINAL-v3-REAL.bak`:

```
[binary file — cannot display]
```

`cat core`:

```
[core dump — process 'cc1' signal 11 (segmentation fault)]
```

`cat nroff-err.log`:

```
nroff: warning: can't find font 'HR'
nroff: warning: can't break line 47
nroff: 2 warnings, 0 errors (but it looks wrong anyway)
```

`cat .Xauthority`:

```
[binary file — cannot display]
```

(X Window System auth file on a text-only Minix machine. A small joke.)

### C3 · `/usr/ast/README`

```
MINIX is for teaching, not production.
If you want to write a real OS, start from scratch.
Good luck.
    -- ast, 1991
```

Foreshadows the Tanenbaum-Torvalds debate (1992). Tanenbaum wrote this
before he knew what Linus was about to do.

### C4 · `/usr/ast/.plan`

`cat /usr/ast/.plan`:

```
Working on MINIX 2.0 filesystem improvements.
Teaching OS course next semester.
Conference paper due October.
```

UNIX `.plan` files — used by `finger`. Adds human texture.

### C5 · `.bash_history`

Shows a previous (failed) session:

```
ls
cd linux-0.01
ls -la
cat RELNOTES-0.01
ftp nic.funet.fi
ascii
put linux-0.01.tar.Z
quit
```

The player sees someone already tried — in ascii mode. Implicit hint:
that attempt didn't work. The quest isn't done.

### C6 · Mail from Tanenbaum

Second mail entry (in addition to existing `mail/welcome.txt`):

`mail/ast.txt`:

```
From: ast@cs.vu.nl (Andy Tanenbaum)
Date: Tue, 16 Sep 1991 14:22:00 +0200
Subject: re: uploading to funet

Linus,

If you're uploading to funet, remember: compressed
archives must transfer in binary mode. ASCII will
corrupt them. I've seen students make this mistake
every semester.

    -- ast
```

Direct hint in-world. The player can find it via `cat mail/ast.txt` or by
following the "you have 2 new message(s)" prompt.

### C7 · `man` command

**Class:** `ManCommand : ICommand`

Supports topics: `ftp`, `ls`, `cat`, `cp`. Others: `No manual entry for <topic>.`

`man ftp` output:

```
FTP(1)                  MINIX Programmer's Manual                 FTP(1)

NAME
    ftp - file transfer program

SYNOPSIS
    ftp [host]

DESCRIPTION
    ftp is the user interface to the Internet standard File
    Transfer Protocol.

COMMANDS
    open host       connect to remote host
    close           close connection
    bye             exit ftp
    ls              list remote directory
    cd dir          change remote directory
    put file        send file to remote
    get file        receive file from remote
    binary          set binary transfer mode
    ascii           set ascii transfer mode
    status          show current status
    help            show command list

TRANSFER MODES
    ascii       Text mode. Line endings are converted.
                Suitable for plain text files only.

    binary      Image mode. No conversion performed.
                REQUIRED for compressed, executable, or
                archive files (.Z, .tar, .gz, .a, .out).

                WARNING: transferring binary files in ascii
                mode WILL corrupt the data.

SEE ALSO
    ftpd(8), netstat(1)

MINIX 1.1                  Sep 1991                              FTP(1)
```

This is the strongest direct hint. Exists for players who are stuck.

### C8 · `/var/log/messages` (system log)

```
Sep 17 21:00:01 kruuna kernel: MINIX 1.1 boot
Sep 17 21:00:01 kruuna kernel: memory: 4096K total, 109K kernel, 3987K free
Sep 17 21:00:02 kruuna init: system startup
Sep 17 21:00:02 kruuna cron: started
Sep 17 21:00:03 kruuna getty: tty0 ready
Sep 17 21:12:00 kruuna login: linus logged in on tty0
Sep 17 21:12:00 kruuna login: session opened for ast on tty1
Sep 17 21:12:00 kruuna login: session opened for (unknown) on tty2
```

That last line. "(unknown)" got a session. When? How? The kernel accepted it.
No further entries about tty2. It's just... there.

---

## D) Easter Eggs & Spooky Responses

### D1 · Standard easter eggs (one-liners)

Each implemented as `IEasterEgg` in a `EasterEggRegistry`:

| Input                   | Response                                                  |
| ----------------------- | --------------------------------------------------------- |
| `emacs`                 | `emacs: not installed. only vi available on this system.` |
| `vi`                    | _(200ms delay)_ `vi: insufficient memory`                 |
| `rm` (anything)         | `rm: permission denied (nice try)`                        |
| `su`                    | `su: incorrect password`                                  |
| `su` (if difficulty=SU) | `su: you chose this name, didn't you?`                    |
| `shutdown`              | `shutdown: must be superuser.`                            |
| `make`                  | `make: no targets. nothing to do.`                        |
| `gcc`                   | `gcc: not installed. try Amsterdam Compiler Kit.`         |
| `reboot`                | `reboot: must be superuser.`                              |
| `finger ast`            | Shows Tanenbaum's `.plan` file (see C4)                   |
| `finger linus`          | `Login: linus  Name: Linus Torvalds  Home: /home/linus`   |
| `finger tty2`           | `finger: tty2: no such user.`                             |
| `echo hello`            | `hello`                                                   |
| `cat /dev/null`         | _(no output — correct behavior)_                          |
| `cat /dev/random`       | `[random bytes — display corrupted]` then garbled chars   |

### D2 · `minix` — the silent one

```
$ minix
$
```

Nothing. No error. No "command not found". No output at all.
Just silently returns to prompt. As if the shell recognized it
but chose not to respond.

If the player types `minix` 3 times in one session:

```
$ minix
$ minix
$ minix
minix: I know.
$
```

Then never responds again. The counter doesn't reset.

**Implementation:** Stateful `IEasterEgg`. Counter in session state.

### D3 · `linux` — the prophecy

`linux` alone:

```
linux: command not found (not yet)
```

Cute. The "(not yet)" is the joke.

But `linux --help`:

```
linux: command not found (not yet)

...but since you asked:

  1. there are files in ~/linux-0.01/
  2. one of them needs to reach nic.funet.fi
  3. ftp is how files travel
  4. compressed archives are not text
  5. the default mode is wrong

good luck.
```

Complete walkthrough disguised as a help page from a program
that doesn't exist yet. Breaks the fourth wall gently.
This is the ultimate stuck-player lifeline.

**Implementation:** `IEasterEgg` checking for `--help` flag.

### D4 · `cat /proc/version` — the glitch

```
Minix version 1.1 (Sep 17 1991) gcc 1.37.1
```

Normal. Except ~5% of the time (after anomaly pings), appends:

```
Minix version 1.1 (Sep 17 1991) gcc 1.37.1
Li
```

Just "Li". Truncated. As if something tried to overwrite the version
string but was cut off. Gone on next `cat`.

### D5 · `dmesg` — kernel ring buffer

**Class:** `DmesgCommand : ICommand`

```
MINIX 1.1 boot
memory: 4096K total, 109K kernel, 3987K free
hd driver: winchester, 40960K
clock: 100 Hz tick
tty: 3 virtual consoles
ethernet: NE2000 at 0x300, IRQ 9
root filesystem: /dev/hd1 (minix)
/usr filesystem: /dev/hd2 (minix)
init: starting /etc/rc
```

Perfectly realistic Minix kernel log.

**Spooky addition** (only after all 3 anomaly pings):
A final line appears:

```
[????] process 0: unnamed: started
```

Process 0 is the kernel itself. It can't "start" an unnamed process.
The brackets should have a timestamp but show only question marks.
This line wasn't there before. The player is sure of it.

---

## E) The Hint Escalation Ladder

Players discover hints in natural exploration order:

| Level | Source              | Directness | How player finds it                                      |
| ----- | ------------------- | ---------- | -------------------------------------------------------- |
| 1     | `.bash_history`     | Indirect   | `cat .bash_history` — see failed ascii attempt           |
| 2     | `mail/ast.txt`      | Moderate   | "2 new messages" prompt → `cat mail/ast.txt`             |
| 3     | `linux-0.01/README` | Moderate   | `cd linux-0.01 && cat README` — upload instructions      |
| 4     | `man ftp`           | Direct     | Curious player checks manual — TRANSFER MODES section    |
| 5     | `linux --help`      | Explicit   | Desperate player types `linux --help` — full walkthrough |

Each level is reachable through natural curiosity. No hint is forced.

---

## F) Implementation Priority

Ordered by atmosphere-per-effort ratio:

| Priority | Feature                                               | Effort | Impact                               |
| -------- | ----------------------------------------------------- | ------ | ------------------------------------ |
| 1        | `IExternalServer` + `NetworkRegistry`                 | Medium | Foundation for ping + future network |
| 2        | `PingCommand` + anomalies                             | Medium | Strongest unique feature             |
| 3        | `IEasterEgg` registry + all one-liners                | Low    | High charm, trivial code             |
| 4        | `minix` (silent/stateful) + `linux --help`            | Low    | Memorable moments                    |
| 5        | `.bash_history` + `mail/ast.txt`                      | Low    | Quest hint infrastructure            |
| 6        | `man ftp`                                             | Low    | Stuck-player safety net              |
| 7        | `/etc/motd`                                           | Low    | Login polish                         |
| 8        | `fortune`, `uptime`, `date`, `uname`, `whoami`, `who` | Low    | Env commands batch                   |
| 9        | Filesystem extras (passwd, tmp, ast, /proc/version)   | Low    | Exploration rewards                  |
| 10       | `dmesg` + `/var/log/messages`                         | Low    | Deep lore for thorough players       |
| 11       | Disk seek flavor                                      | Low    | Nice-to-have polish                  |
| 12       | `date` glitch + `/proc/version` glitch                | Low    | Post-anomaly spooky payoff           |

---

## G) Design Rules

1. **Realism first.** Everything should look like real Minix 1.1 output.
   The spooky elements hide inside that realism.

2. **Restraint.** Exactly 3 anomaly hosts. Exactly 1 unnamed user on tty2.
   The `minix` command speaks exactly once. Mystery comes from scarcity.

3. **No jump scares.** Nothing loud, nothing sudden. Just quiet wrongness.
   A line that shouldn't be there. A log entry with no timestamp.
   The player notices — or doesn't. Both are fine.

4. **Hints never punish.** Every hint is opt-in. The player who reads
   `man ftp` isn't penalized. The player who types `linux --help`
   gets help without judgment.

5. **State-driven spookiness.** The weird stuff escalates after anomaly pings.
   Players who don't ping google/github/wikipedia see a normal system.
   The anomalies are the trigger. Curiosity unlocks the mystery layer.

6. **Each application = `IApplication` class.** No exceptions.
   Shell, FTP, future mail client, future editor — all go through the stack.

7. **Each server = `IExternalServer`.** All network hosts are registered objects.
   DNS, ping, FTP connection all resolve through `NetworkRegistry`.

8. **Easter eggs never block progress.** One line, move on. Never a dead end.
