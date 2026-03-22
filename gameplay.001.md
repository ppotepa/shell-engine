# GAMEPLAY.001 — Shell Quest: Prologue

## The Simulation

Shell Quest is a terminal-based educational RPG. The player sits in front of a
simulated CRT monitor running a faithful reproduction of **MINIX 1.1** — the
real operating system created by Andrew S. Tanenbaum in 1987, as it would have
existed on a Finnish university workstation in **September 1991**.

Everything the player sees, types, and experiences passes through a simulated
operating system (the "cognitos-os" sidecar). The game engine renders the
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

The player is **Linus Torvalds**, 21 years old, second-year CS student.
He has just finished the first version of his hobby operating system project
(Linux 0.01) and needs to upload it to the public FTP archive at
nic.funet.fi.

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

1. Player boots machine → MINIX 1.1 loads (animated, services start)
2. Login as `linus` → create password on first boot
3. Explore filesystem → find linux-0.01/ with the source archive
4. Read notes, mail → hints about FTP and binary mode
5. `ftp nic.funet.fi` → connect, navigate to /pub/OS/Linux
6. `put linux-0.01.tar.Z` → **fails** (ASCII mode corrupts binary)
7. Discover the problem → `binary` → re-upload → **success**

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

### What the player sees after login

```
account created.
last login: Tue Sep 17 21:12
you have 2 new messages.
type ls to look around.
type cat <file> to read notes.

linus@kruuna:~ [0]$ _
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

```
/home/linus/
├── linux-0.01/
│   ├── RELNOTES-0.01          ← Linus's release notes
│   ├── linux-0.01.tar.Z       ← THE file to upload (73K compressed)
│   ├── bash.Z                 ← shell binary
│   ├── update.Z               ← update daemon
│   └── README                 ← build instructions + FTP hint
├── mail/
│   ├── welcome.txt            ← from Operator
│   └── ast.txt                ← from Tanenbaum (binary mode hint)
├── notes/
│   └── starter.txt            ← basic commands cheatsheet
├── .bash_history              ← previous (failed) FTP session
├── .profile                   ← PATH, TERM, EDITOR
└── .plan                      ← "working on something"

/etc/
├── passwd                     ← 6 users (root, daemon, bin, ast, linus, nobody)
├── hostname                   ← kruuna
├── hosts                      ← DNS table (4 entries)
├── services                   ← port assignments (ftp, telnet, smtp, http, finger)
├── resolv.conf                ← nameserver 128.214.1.1
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

/var/log/
├── messages                   ← syslog (grows over time)
├── cron.log                   ← cron execution log
├── auth.log                   ← login records (tty2 anomaly here)
└── net.trace                  ← appears after anomaly pings

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

- `.bash_history` shows a previous **failed** FTP attempt (ascii mode)
- `mail/ast.txt` from Tanenbaum explicitly mentions binary mode
- `/var/log/auth.log` shows tty2 login with timestamp `Jan 1 00:00:00`
- `/tmp/core` — a segfault dump from cc1 (the C compiler)
- `usr/ast/minix-2.0-notes.txt` — Tanenbaum's private design notes
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

## What Needs Better Realistic Styling

### Currently shallow → needs depth

1. **Boot sequence** — should feel like real hardware POST + service init.
   Each line should come from a real Kernel subsystem, not hardcoded strings.

2. **`ps` / `top`** — processes are 4 hardcoded entries with sinusoidal CPU.
   Should show real process tree: init→netd→maild→sh→(current command).

3. **`ls` output** — just shows filenames. Real Minix shows:
   ```
   -rw-r--r--  1 linus  users  73091 Sep 17 21:00  linux-0.01.tar.Z
   drwxr-xr-x  2 linus  users    512 Sep 17 20:45  notes/
   ```
   Needs: permissions, owner, group, size, date.

4. **Services** — `netd`, `maild`, `crond` are names only. Should tick,
   produce logs, generate mail, rotate files. A living system.

5. **Pipes** — `cat /var/log/messages | grep anomaly` doesn't work.
   Essential Unix operation. Shell must parse `|` and chain stdout→stdin.

6. **Environment variables** — `echo $HOME` doesn't expand. `export` doesn't
   exist. Player can't modify PATH or set variables.

7. **Journal/Logging** — `/var/log/messages` is static seed text. Should grow
   over time as syslogd aggregates from services and kernel events.

8. **File metadata** — no permissions, no timestamps, no sizes. VFS stores
   content only. Need `FileStat` with mode/owner/group/size/mtime.

9. **Process lifecycle** — commands don't spawn processes. `kill` can't work.
   Boot doesn't actually start services as processes.

10. **Time awareness** — `date` shows epoch but uptime doesn't persist across
    sessions. Login delta not tracked. No "system was down for 8 hours".
