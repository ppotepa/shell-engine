# GAMEPLAY.001 — Shell Quest: Prologue

## The Simulation

Shell Quest is a terminal-based educational RPG. The player sits in front of a
simulated CRT monitor running a faithful reproduction of **MINIX 1.1** — the
real operating system created by Andrew S. Tanenbaum in 1987, as it would have
existed on a Finnish university workstation in **September 1991**.

Everything the player sees, types, and experiences passes through a simulated
operating system (the "cognitOS" sidecar). The game engine renders the
terminal as a 3D object; the OS simulation runs as a separate C# process
communicating over stdin/stdout JSON IPC.

The simulation is not a shell skin. It is a **living system**: processes run,
services tick, logs grow, mail arrives, time passes, disk fills up. The player
doesn't interact with a script — they interact with an OS.

---

## Historical Setting: September 17, 1991

### Where

**kruuna.helsinki.fi** — a shared workstation at the University of Helsinki,
Department of Computer Science. A 386 or 486 box (depending on difficulty)
running MINIX 1.1 with a 3Com Ethernet card on FUNET (Finnish University
and Research Network).

### Who

The player is **Linus Benedict Torvalds** (login: `torvalds`), 21 years old, 
second-year CS student. He has just finished the first version of his hobby 
operating system project (Linux 0.01) and needs to upload it to the public 
FTP archive at nic.funet.fi.

Three weeks earlier (August 25, 1991), he posted his famous message to
comp.os.minix: *"Hello everybody out there using minix — I'm doing a (free)
operating system (just a hobby, won't be big and professional like gnu)..."*

### What exists in 1991

| Technology        | Status                                              |
|-------------------|-----------------------------------------------------|
| MINIX             | Version 1.1, widely used for teaching               |
| FTP               | Primary way to distribute software (since 1971)     |
| Email (SMTP)      | University-to-university, no web mail               |
| Usenet            | comp.os.minix, alt.*, rec.* — the "social media"   |
| finger            | Check who's logged in on remote hosts               |
| telnet            | Remote login (no SSH yet — that's 1995)             |
| DNS               | Working since 1983, hierarchical                    |
| Gopher            | Just released (spring 1991) — menu-based info       |
| World Wide Web    | Tim Berners-Lee announced it Aug 6, 1991            |
|                   | Exists on 1 server (info.cern.ch). Almost nobody    |
|                   | has heard of it. No "browsers" as we know them.     |
| IRC               | Exists since 1988, mainly Finnish universities      |
| NFS               | Network file sharing, mainly Sun workstations       |
| X11               | Windowing system — NOT on MINIX (too heavy)         |
| gcc               | Version 1.37 — the C compiler Linus uses            |
| vi / ed           | The text editors available                           |

### What does NOT exist in 1991

| Technology     | When it appeared | Notes                            |
|----------------|-----------------|----------------------------------|
| Linux          | THIS DAY        | 0.01 exists on Linus's disk only |
| SSH            | 1995            | People use telnet/rlogin         |
| Web browsers   | 1993 (Mosaic)   | WWW is a CERN curiosity          |
| Google         | 1998            | —                                |
| Wikipedia      | 2001            | —                                |
| GitHub         | 2008            | —                                |
| HTTP servers   | ~1991           | 1 exists (info.cern.ch)          |
| Consumer ISPs  | ~1994           | Internet = universities/military |
| MP3            | 1993            | —                                |
| JPEG           | 1992            | —                                |
| USB            | 1996            | —                                |

---

## Core Gameplay Loop

```
┌─────────────────────────────────────────────────┐
│  BOOT → LOGIN → EXPLORE → DISCOVER → SOLVE     │
│                    ↑                    │        │
│                    └────────────────────┘        │
│                                                  │
│  Side loops:                                     │
│    • Read mail/notes → get hints                 │
│    • Discover anomalies → unlock mystery thread  │
│    • Try forbidden commands → find easter eggs   │
│    • Watch system logs → notice strange things   │
└─────────────────────────────────────────────────┘
```

### Primary Quest: The Upload

1. Player boots machine → MINIX 1.1 loads (animated boot sequence, services start)
2. Login as `torvalds` → create password on first boot (max 5 characters)
3. Start in `/usr/torvalds/` → explore filesystem with `ls`, `cd`, `cat`
4. Find `linux-0.01/` directory with the source archive
5. Read `mail/` folder (1 new message) → hints about FTP and binary mode
6. Read `notes/starter.txt` → basic command cheatsheet
7. Check `.sh_history` → shows previous **failed** FTP attempt (uploaded in ASCII mode)
8. `ftp nic.funet.fi` → connect, navigate to /pub/OS/Linux
9. `put linux-0.01.tar.Z` → **fails** (ASCII mode corrupts binary)
10. Discover the problem → `binary` → re-upload → **success**

### Secondary Thread: The Anomalies

The machine is slightly wrong. Not broken — just... aware of things it
shouldn't be. This thread is entirely optional but rewards curiosity:

- `ping google.com` → mysterious DNS error (Google doesn't exist yet)
- `ping github.com` → partial route data, IANA allocation unknown
- `ping en.wikipedia.org` → temporal routing anomaly
- `/var/log/net.trace` appears and grows with each anomaly
- `dmesg` shows `[????] process 0: unnamed: started` after 3 anomalies
- `netstat` shows an UNKNOWN connection on an impossible port
- `date` occasionally glitches after anomaly exposure
- The anonymous user on tty2 (`who` command) has no login timestamp

None of these block progress. They're breadcrumbs for Act 2.

---

## The Simulated Environment

### What the player can do

**Navigation:**
- `ls`, `cd`, `pwd` — browse a ~60 file VFS with /home, /etc, /usr, /tmp, /var, /dev, /proc
- `cat`, `head`, `tail` — read files (notes, mail, configs, logs)
- `cp` — copy files (backup before upload)
- `grep` — search file contents

**System inspection:**
- `ps`, `top` — running processes (dynamic, CPU varies)
- `services` — running daemons (netd, maild, crond, syslogd)
- `df`, `free`, `mount` — disk, memory, filesystems
- `dmesg` — kernel ring buffer (grows with anomalies)
- `uptime`, `date`, `uname`, `hostname` — system identity
- `who`, `whoami`, `id`, `finger` — user information
- `env`, `echo`, `history` — shell environment

**Network:**
- `ping <host>` — 13 resolvable hosts (8 real 1991 + 2 loopback + 3 anomaly)
- `nslookup <host>` — DNS lookup
- `netstat` — active connections and listening ports
- `ifconfig` — network interface (3Com EtherLink, 10Mbps)
- `ftp <host>` — full FTP client (open, binary, ascii, put, ls, cd, pwd, bye)

**Discovery:**
- `man <topic>` — manual pages (ftp, ls, cat, cp, ping, chmod, hier)
- `help` — command list
- `fortune` — random quotes (1 in 10 is unsettling)
- `finger <user>` — check user info (linus, ast, root, tty2 mystery)
- Easter eggs: `minix`, `linux`, `emacs`, `vi`, `make`, `su`, `halt`, etc.

### What the player sees on boot

```
MINIX 1.1 boot
memory: 4096K total, 109K kernel, 3987K free
hd driver: winchester, 20480K
clock: 100 Hz tick
tty: 3 virtual consoles
ethernet: 3Com EtherLink II at 0x300, IRQ 9    [OK]
root filesystem: /dev/hd1 (minix)              [OK]
/usr filesystem: /dev/hd2 (minix)              [OK]
init: starting /etc/rc
starting netd...                               [OK]
starting maild...                              [OK]
starting cron...                               [OK]

Minix 1.3  Copyright 1987, Prentice-Hall
Console ready

kruuna login: _
```

MINIX, version, and OK markers are brightly colored. Hardware specs
come from `MachineSpec` (varies by difficulty).

### What the player sees after login (first boot)

```
account created.
last login: Tue Sep 17 21:12
you have 1 new message.
type ls to look around.
type cat <file> to read notes.

torvalds@kruuna:/usr/torvalds [0]$ _
```

On subsequent logins:
```
last login: Tue Sep 17 21:12
you have 1 new message.
type ls to look around.
type cat <file> to read notes.

torvalds@kruuna:/usr/torvalds [0]$ _
```

---

## Difficulty Model

Difficulty affects hardware specs, which affect simulation speed and hints:

| Difficulty       | CPU           | RAM    | NIC        | FTP Hints        |
|------------------|---------------|--------|------------|------------------|
| MOUSE ENJOYER    | 486 DX2-66    | 8192K  | 10000 Kbps | Clear warning    |
| SCRIPT KIDDIE    | 486 SX-25     | 6144K  | 4800 Kbps  | Moderate hint    |
| I CAN EXIT VIM   | 386 DX-33     | 4096K  | 2400 Kbps  | Vague hint       |
| DVORAK           | 386 SX-25     | 2048K  | 1200 Kbps  | Very vague       |
| SU               | 386 SX-16     | 1024K  | 300 Kbps   | No hint at all   |

Slower NIC = longer FTP transfer simulation. Less RAM = `free` shows tighter.
CPU model appears in `dmesg` and `uname`.

---

## Filesystem Map (September 17, 1991)

**VFS Root = `/usr/torvalds/`** (player's home directory)

```
/usr/torvalds/
├── linux-0.01/
│   ├── RELNOTES-0.01          ← Linus's release notes
│   ├── linux-0.01.tar.Z       ← THE file to upload (73K compressed)
│   ├── bash.Z                 ← shell binary
│   ├── update.Z               ← update daemon
│   └── README                 ← build instructions + FTP hint
├── mail/
│   ├── welcome.txt            ← from Operator (unread on first boot)
│   └── ast.txt                ← from Tanenbaum (binary mode hint)
├── notes/
│   └── starter.txt            ← basic commands cheatsheet
├── .sh_history                ← previous (failed) FTP session
├── .profile                   ← PATH, TERM, EDITOR
└── .plan                      ← "working on something"

/etc/
├── passwd                     ← 5 users (root, daemon, bin, ast, torvalds)
├── group                      ← groups (operator, staff, other)
├── hostname                   ← kruuna
├── hosts                      ← DNS table
├── services                   ← port assignments (ftp, telnet, smtp, http, finger)
├── resolv.conf                ← nameserver 128.214.1.1
├── motd                       ← message of the day
└── rc                         ← init script

/usr/ast/                      ← Tanenbaum's home directory
├── README                     ← "MINIX is for teaching"
├── .plan                      ← working on MINIX 2.0
└── minix-2.0-notes.txt        ← design notes (draft)

/tmp/
├── thesis-FINAL-v3-REAL.bak   ← binary (every university has this)
├── core                       ← core dump from cc1 (segfault)
├── nroff-err.log              ← formatter warnings
├── .Xauthority                ← binary
└── .lock-ast                  ← empty lockfile

/usr/adm/  (system logs - MINIX 1.1 uses /usr/adm not /var/log)
├── messages                   ← syslog (grows over time)
├── cron                       ← cron execution log
└── wtmp                       ← login records

/proc/
└── version                    ← Minix version 1.1

/dev/
├── null                       ← empty
├── random                     ← corrupted display
└── console                    ← device stub

/usr/bin/  /usr/lib/  /usr/man/  /usr/src/minix/  /bin/
```

---

## Network Map (September 1991)

### Real hosts (pingable, resolvable)

| Host                       | IP             | Ping   | Role                        |
|----------------------------|----------------|--------|-----------------------------|
| localhost                  | 127.0.0.1      | 0.01ms | Loopback                    |
| kruuna                     | 127.0.0.1      | 0.01ms | Local machine               |
| nic.funet.fi               | 128.214.6.100  | 12ms   | Finnish FTP archive (target)|
| ftp.funet.fi               | 128.214.6.100  | 12ms   | Alias for nic.funet.fi      |
| cs.vu.nl                   | 130.37.24.3    | 45ms   | VU Amsterdam (Tanenbaum)    |
| sun.nl                     | 192.16.184.1   | 48ms   | Sun Microsystems Netherlands|
| ftp.gnu.org                | 128.52.14.10   | 180ms  | GNU project FTP             |
| nntp.funet.fi              | 128.214.6.101  | 14ms   | Usenet news server          |
| ftp.uu.net                 | 137.39.1.2     | 210ms  | UUNET archive               |
| info.cern.ch               | 128.141.201.74 | 62ms   | THE web server (only one!)  |

### Anomaly hosts (temporal impossibilities)

| Host              | Response                                                   |
|-------------------|------------------------------------------------------------|
| google.com        | DNS resolves → route incomplete → "host allocation: IANA RESERVED" |
| github.com        | Partial route → "route fragments suggest future allocation"        |
| en.wikipedia.org  | Route analysis → "temporal inconsistency in routing table"         |

Each anomaly ping appends to `/var/log/net.trace` and tracks in QuestState.

---

## Easter Eggs & Hidden Content

### Command easter eggs

| Input         | Response                                                |
|---------------|---------------------------------------------------------|
| `minix`       | Silent (×2), then `I know.` on 3rd try                |
| `linux`       | `linux: command not found yet`                          |
| `linux --help`| Full walkthrough of the prologue quest                  |
| `emacs`       | `emacs: not enough memory (need strStrtime virtual)`   |
| `vi`          | `vi: file not specified. (use ed instead, it's better)` |
| `make`        | `make: no targets specified and no makefile found.`     |
| `su`          | `su: incorrect password` (always)                       |
| `halt`        | `halt: must be superuser.`                              |
| `gcc`         | `gcc: no input files`                                   |
| `rm -rf /`    | `rm: cannot remove '/': Permission denied (nice try)`  |
| `exit`        | `logout: where would you go?`                           |
| `passwd`      | `passwd: only root may change passwords`                |
| `shutdown`    | `shutdown: must be superuser.`                          |
| `reboot`      | `reboot: must be superuser.`                            |

### Filesystem discoveries

- `.sh_history` shows a previous **failed** FTP attempt (ascii mode)
- `mail/ast.txt` from Tanenbaum explicitly mentions binary mode
- `/usr/adm/wtmp` — login records (inspect with custom tools)
- `/tmp/core` — a segfault dump from cc1 (the C compiler)
- `/usr/ast/minix-2.0-notes.txt` — Tanenbaum's private design notes
- `/dev/random` displays corrupted characters

### Fortune quotes (1 in 10 is spooky)

Normal:
- "langstroth said: a program should do one thing and do it well."
- "strstrtime is the root of all evil. — knuth"
- etc.

Spooky (selected randomly when anomaly count > 0):
- "the machine hums. it's not the fan."
- "you weren't the first to log in today."

---

## Current Implementation Status

### ✅ Fully Implemented (MINIX 1.1 Accurate)

1. **Boot sequence** — Real kernel subsystem boot with hardware detection:
   - Memory detection and kernel/free memory reporting
   - Winchester disk driver initialization
   - Clock tick configuration (100 Hz)
   - Virtual console setup (3 TTYs)
   - Ethernet card detection (3Com EtherLink II)
   - Root and /usr filesystem mount
   - Service startup (netd, maild, cron)

2. **`ls` output** — Full MINIX-style long format implemented:
   ```
   -rw-rw-r--  1 torvalds  staff  73091 Sep 17 21:00  linux-0.01.tar.Z
   drwxr-xr-x  2 torvalds  staff    512 Sep 17 20:45  notes/
   ```
   Shows: permissions, link count, owner, group, size, mtime

3. **File metadata** — Complete `InodeTable` with UNIX semantics:
   - Mode bits (rwxrwxrwx + type)
   - Owner/group resolution via /etc/passwd and /etc/group
   - Link counts
   - Timestamps (mtime, ctime) seeded from epoch (Sep 17, 1991 21:00 UTC)

4. **User authentication** — Real /etc/passwd parsing:
   - First boot: create account as `torvalds` with password (max 5 chars)
   - Subsequent logins: password verification
   - Home directory resolution from passwd entry
   - Shell assignment per user

5. **Mail system** — Simulated mail spool with disk I/O:
   - `/var/spool/mail/torvalds` backing store
   - Unread count shown at login
   - Mail delivery via `IMailSpool` interface
   - Read/unread tracking

6. **Network simulation** — Real DNS resolution and ping timing:
   - 10 historically accurate 1991 hosts (nic.funet.fi, cs.vu.nl, etc.)
   - 3 anomaly hosts (google.com, github.com, wikipedia.org)
   - Realistic RTT simulation based on geographic distance
   - FTP client with ASCII/binary mode switching

7. **Services** — Living daemons that tick and produce logs:
   - `netd` — network daemon
   - `maild` — mail delivery daemon
   - `crond` — cron scheduler
   - All tracked as simulated processes with CPU usage

8. **Process simulation** — Full process table:
   - init (PID 1) → service tree
   - Dynamic process spawning for commands
   - CPU usage tracking with realistic variance
   - `ps` and `top` show real process state

### 🚧 Partially Implemented

9. **Pipes and redirection** — Shell pipeline parsing implemented:
   - Command chaining with `|` works
   - Stdout→stdin piping functional
   - Redirection (`>`, `>>`, `<`) may have gaps

10. **Environment variables** — Partial support:
    - `$HOME`, `$PATH`, `$USER` readable
    - `export` and shell variable assignment need expansion

### 📋 Not Yet Implemented

11. **Dynamic logging** — `/usr/adm/messages` is seeded but static:
    - Should grow as services generate events
    - syslogd aggregation not yet wired to service ticks

12. **Time persistence** — `date` shows epoch but:
    - Uptime doesn't persist across game sessions
    - "System was down for X hours" not tracked
