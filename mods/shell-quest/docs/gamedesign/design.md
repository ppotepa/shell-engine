# Shell Quest — Game Design Document

## 1. Design Philosophy

### 1.1 Three pillars

**Authenticity** — Everything the player sees could plausibly appear on a real
MINIX 1.1 system in September 1991. File formats, command outputs, error
messages, timestamps, network topology — all historically researched. The
simulation is not a parody or simplification. It is a faithful reproduction
with a story woven into its seams.

**Discovery** — The game never tells the player what to do explicitly. There
is no tutorial popup, no quest marker, no glowing objective. Information
exists in the filesystem: mail, notes, history, man pages, configs, logs.
The player learns by reading, experimenting, and connecting information
from multiple sources. This mirrors how real Unix users learned in 1991.

**Quiet wrongness** — Beneath the authentic surface, something is slightly
off. Not broken, not glitchy — just wrong in ways that reward attention.
A user logged in at epoch zero. Network routes to hosts that won't exist
for years. A process with no name. The game never explains these. They
accumulate silently, building unease that pays off in later acts.

### 1.2 What this is NOT

- Not a typing game (speed doesn't matter)
- Not a puzzle game (there's one puzzle per act, not dozens)
- Not a horror game (unsettling, not scary)
- Not a Linux tutorial (it teaches Unix concepts through immersion)
- Not a visual novel (no character portraits, no dialogue trees)

---

## 2. Target Experience

### 2.1 First 5 minutes

Player selects difficulty from 3D character portraits. Screen goes black.
MINIX boots — hardware detection, services starting, green OK markers.
Login prompt appears. Player types `linus`, creates a password. Shell opens.

**Feeling:** "I'm actually using a computer from 1991."

### 2.2 Minutes 5–15

Player explores. `ls` shows files. `cat mail/welcome.txt` — a terse welcome.
`cat mail/ast.txt` — Tanenbaum talks about binary mode. Player browses
linux-0.01/, reads the README, maybe checks `top` or `ps`. Tries some
commands. Discovers `fortune`, reads .bash_history.

**Feeling:** "This is a real system. These files make sense. I'm learning."

### 2.3 Minutes 15–25

Player connects to FTP, tries to upload. Fails (ASCII mode). Re-reads
the clues. Tries `binary`. Uploads. Succeeds.

**Feeling:** "I just did what Linus did. I uploaded Linux to the internet."

### 2.4 Optional: the curious player

Some players will `ping google.com`. See the strange error. Ping more
future hosts. Read the growing net.trace. Notice the tty2 user. Read
auth.log. Check dmesg after 3 anomalies. Try the `minix` command three
times. Type `linux --help`.

**Feeling:** "Wait... what is going on with this machine?"

---

## 3. Realism Audit — September 1991

Every element must pass this test: **"Would this exist on kruuna.helsinki.fi
on September 17, 1991?"**

### 3.1 Operating System

**MINIX 1.1** (Andrew S. Tanenbaum, 1987)
- Microkernel architecture
- Ran on IBM PC, 286/386
- Used for teaching operating systems at universities
- Version numbering: "1.1" is the kernel, "1.3" is the distribution
  (source of the "Minix 1.3 Copyright 1987, Prentice-Hall" banner)
- Shell: Bourne shell (/bin/sh), not bash
- Editor: ed (line editor), possibly vi
- Compiler: gcc 1.37 (Amsterdam Compiler Kit also available)
- No X11 (too heavy for MINIX)

**Historically accurate commands:**
ls, cat, cp, mv, rm, mkdir, rmdir, cd, pwd, echo, head, tail, grep, wc,
sort, uniq, chmod, chown, date, who, whoami, uname, hostname, finger, ps,
kill, df, du, mount, umount, sync, dd, tar, compress, uncompress, man,
more, less (maybe), find, diff, comm, ed, vi, cc/gcc, make, ld, ar, nm,
strip, ftp, telnet, finger, ping, nslookup, mail, cron, at, sh

**Commands that should NOT exist:**
ssh (1995), wget (1996), curl (1997), apt/yum (late 90s), git (2005),
python (1991 but not on MINIX), perl (existed but unlikely on MINIX),
top (existed on BSD but custom for MINIX), nano (1999), sudo (existed
but unlikely on MINIX)

Note: We include `top` as a gameplay convenience despite questionable
historical accuracy. It's too useful for system monitoring to omit.

### 3.2 Network

**FUNET** (Finnish University and Research Network)
- Connected to NORDUnet (Nordic) → NSFNET (US backbone)
- Speed: 64 Kbps international link (1991)
- Protocol: TCP/IP over Ethernet
- DNS: hierarchical, working
- No firewalls (concept barely existed)
- No NAT (every machine had a real IP)

**Services available:**
- FTP (primary software distribution method)
- SMTP email (university-to-university)
- Usenet/NNTP (newsgroups — the "social media")
- finger (check if someone is online)
- telnet (remote login — no SSH yet)
- Gopher (just launched, menu-based information system)
- HTTP: Tim Berners-Lee's server at CERN (info.cern.ch)
  running since late 1990. Almost nobody knows about it.

**What does NOT work:**
- HTTP as we know it — there's 1 web server in the world
- DNS for any domain registered after 1991
- Any service that requires modern TLS/SSL

### 3.3 The Machine: kruuna.helsinki.fi

**kruuna** is a real hostname from Helsinki CS department history.
- IP: 130.234.48.x (Helsinki university range)
- Hardware: 386/486 depending on difficulty
- Ethernet: 3Com EtherLink II (ISA, 10 Mbps)
- Disk: Winchester (IDE) hard drive
- Display: VGA text mode 80×25 (we extend to terminal size)
- 3 virtual consoles (tty0, tty1, tty2)

### 3.4 People

**Linus Torvalds** (the player character)
- 21 years old, 2nd year CS student
- Posted "Hello everybody..." to comp.os.minix on Aug 25, 1991
- Linux 0.01 completed mid-September 1991
- Uses MINIX as development host
- Knows C, 386 assembly, some Unix administration

**Andrew S. Tanenbaum** (mail sender, directory owner)
- Professor at Vrije Universiteit Amsterdam
- Created MINIX in 1987 for his OS textbook
- Will argue with Linus in January 1992 ("LINUX is obsolstrStrtime")
- In September 1991: working on MINIX 2.0, cordial relationship

**The tty2 user** (mystery)
- No real-world counterpart
- Game fiction: an entity logged in at Unix epoch (Jan 1 00:00:00 1970)
- No explanation given in prologue
- Setup for Act 2 narrative

---

## 4. Simulation Fidelity — What Needs Styling

### 4.1 CRITICAL — must be realistic

| Element | Current State | Required Realism |
|---------|--------------|------------------|
| Boot sequence | Hardcoded text lines | Real service init: spawn process per service, log to journal |
| `ps` output | 4 static entries | Dynamic process tree: init→daemons→shell→command |
| `ls` output | Filename only | Full: `-rwxr-xr-x 1 linus users 73091 Sep 17 21:00 file` |
| `/var/log/messages` | Static seed text | Grows over time as syslogd ticks |
| `services` | Names + "active" | PID, uptime, status, last-checked timestamp |
| `date` output | Epoch + offset | Exact format: `Tue Sep 17 21:15:33 UTC 1991` |
| `env` output | Hardcoded list | Mutable environment (export, unset) |
| Pipes (`\|`) | Not supported | `cat file \| grep pattern \| wc -l` must work |
| Redirects (`>`) | Not supported | `echo text > file` must work |

### 4.2 IMPORTANT — adds depth

| Element | Current State | Required Realism |
|---------|--------------|------------------|
| File permissions | None | Mode bits (0755, 0644), shown in ls -l |
| File sizes | Not tracked | Calculated from content length |
| File timestamps | None | mtime seeded at epoch, updated on write |
| File ownership | None | owner:group (linus:users, root:root, ast:staff) |
| `kill` | Always "denied" | Works on user processes, denied for root |
| FTP DnsTable | In FtpApplication | Should use NetworkStack (shared DNS) |
| Process PIDs | Static | Dynamic allocation, parent-child tree |
| `top` refresh | Static snapshot | Should show changing values on each call |

### 4.3 NICE TO HAVE — polish

| Element | Notes |
|---------|-------|
| `chmod` actually changes permissions | Affects what cat/ls can access |
| Cron log rotation | /var/log/messages trimmed when too large |
| Service crash/restart | After anomalies, services briefly crash |
| `more`/`less` pager | Long file output paged |
| Tab completion | Engine-side input assistance |
| `!!` / `!n` history | Bash-style history expansion |
| `alias` support | User-defined command aliases |
| `sort`, `uniq` | Additional pipe-friendly commands |

---

## 5. Architecture Requirements for Realism

### 5.1 Kernel subsystems (new)

The OS needs to be more than a command dispatcher. It needs **subsystems**
that maintain state independently and interact through the kernel:

```
IKernel
├── IClock          — simulated time, uptime, wall clock
├── IProcessTable   — spawn, kill, list, parent-child tree
├── IServiceManager — register, start, stop, tick each service
├── INetworkStack   — DNS, ping, connections, interfaces
├── IJournal        — kernel ring buffer, syslog, auth log
├── IMailSpool      — deliver, read, unread count
├── IFileSystem     — read, write, stat, chmod, ls -l data
└── IEnvironment    — mutable env vars per session
```

Each subsystem:
- Has its own state
- Ticks independently (called by Kernel.Tick)
- Can access other subsystems through Kernel reference
- Commands access subsystems through `CommandContext.Kernel`

### 5.2 Why this matters for realism

**Without Kernel subsystems:**
- `dmesg` returns a hardcoded list — always the same
- `ps` returns 4 static entries — no connection to what's running
- `netstat` checks QuestState manually — knows about FTP by magic
- Services are names — they don't tick, don't log, don't crash
- Boot is cosmetic — no real initialization happens

**With Kernel subsystems:**
- `dmesg` reads from `Journal.Dmesg()` — ring buffer grows as things happen
- `ps` reads from `Processes.All()` — shows real process tree
- `netstat` reads from `Network.ActiveConnections()` — tracks real state
- Services tick, log to Journal, manage their own PIDs
- Boot calls `Services.Start()` which spawns processes and logs to Journal

### 5.3 Command dependency resolution

**Current** (ad-hoc, constructor injection):
```
PingCommand(NetworkRegistry network)  ← knows about network specifically
DmesgCommand()                        ← reads Quest state directly
NetstatCommand()                      ← reads Quest state directly
```

**Proposed** (unified, through Kernel):
```
PingCommand.Execute(ctx):
    server = ctx.Kernel.Network.Resolve(host)
    ctx.Kernel.Journal.Net($"ping {host}")

DmesgCommand.Execute(ctx):
    lines = ctx.Kernel.Journal.Dmesg()

NetstatCommand.Execute(ctx):
    conns = ctx.Kernel.Network.ActiveConnections()
```

Every command talks to the same Kernel. No special injection needed.
Commands don't know about each other. They observe shared state
through Kernel subsystems.

---

## 6. Content Pipeline

### 6.1 How content is authored

All OS content lives in C# code:
- Commands: `Commands/*.cs` — one class per command
- Applications: `Applications/*.cs` — FTP, future mail client
- Epoch files: `VirtualFileSystem.SeedEpochFiles()` — seeded on load
- Boot sequence: `MinixBootSequence.BuildBootSteps()` — boot lines
- Network: `NetworkRegistry` — host definitions
- Easter eggs: `EasterEggs/*.cs` — special responses

### 6.2 How content reaches the player

```
C# sidecar (cognitos-os)
  ↓ JSON over stdio
Engine (Rust)
  ↓ terminal sprite rendering
3D scene (CRT monitor object)
  ↓ compositor
Player's actual terminal
```

The engine doesn't know about MINIX, FTP, or quests. It just renders
what the sidecar sends. The sidecar doesn't know about 3D, sprites,
or scenes. Clean separation.

### 6.3 Historically accurate content sources

When writing file contents, mail text, error messages, or command output:

1. **Use real Minix/Unix error messages** — not invented ones
   - "No such file or directory" (not "File not found")
   - "Permission denied" (not "Access denied")
   - "command not found" (not "Unknown command")

2. **Use real 1991 conventions** — lowercase, terse, no emoji
   - "3 packets transmitted, 3 received, 0% packet loss"
   - "Connection closed by foreign host."
   - "226 Transfer complete."

3. **Use real formats** — RFC-compliant where applicable
   - FTP responses: 3-digit codes (220, 331, 226, 550)
   - Mail headers: From/Date/Subject RFC 822 format
   - Timestamps: Unix ctime format (`Tue Sep 17 21:12:00 1991`)

4. **Reference real people and places** — but gently
   - Tanenbaum's writing style (direct, professorial)
   - Helsinki university hostnames (kruuna, nic.funet.fi)
   - Real IP ranges (128.214.x.x = FUNET)

---

## 7. Difficulty Design

### 7.1 Philosophy

Difficulty should feel like **different hardware**, not different games.
The same world, same files, same commands. But the machine is faster
or slower, and the system gives more or fewer hints.

### 7.2 The five tiers

| Tier | Name | Machine | Hint Level | Target Player |
|------|------|---------|------------|---------------|
| 1 | MOUSE ENJOYER | 486 DX2-66, 8MB | Explicit | Never used a terminal |
| 2 | SCRIPT KIDDIE | 486 SX-25, 6MB | Clear | Knows `ls` and `cd` |
| 3 | I CAN EXIT VIM | 386 DX-33, 4MB | Moderate | Comfortable with Unix |
| 4 | DVORAK | 386 SX-25, 2MB | Vague | Unix power user |
| 5 | SU | 386 SX-16, 1MB | None | Wants the authentic struggle |

### 7.3 What changes per difficulty

**Hardware (cosmetic + pacing):**
- CPU model → shown in dmesg, uname, boot
- RAM → shown in free, dmesg, top
- NIC speed → affects FTP transfer time
- Disk → shown in df

**Hints (gameplay):**
- Tier 1: FTP failure says "check transfer mode (ascii vs binary)"
- Tier 2: FTP failure says "archive may be damaged, check transfer mode"
- Tier 3: FTP failure says "archive may be damaged"
- Tier 4: FTP failure says "transfer failed"
- Tier 5: FTP failure gives no additional message beyond "226 Transfer complete"

**Boot pacing:**
- Faster CPU → shorter boot delays
- Slower CPU → longer boot (player watches more)

### 7.4 What does NOT change

- Available commands (all 30+ always available)
- Filesystem content (same files, same mail)
- Easter eggs (all always accessible)
- Anomaly behavior (same responses)
- Quest solution (always: binary → put)

---

## 8. Sound Design (Future)

Currently silent. Planned:
- CRT hum (ambient)
- Keypress sounds (mechanical keyboard)
- Hard drive seek on file operations
- Modem/network sounds on FTP connect
- Boot POST beep

All sounds would be era-appropriate: no digital FX, only
analog/mechanical sounds from 1991 hardware.

---

## 9. Accessibility Considerations

- All gameplay is text-based → inherently screen-reader compatible
- Color is used for style, never for essential information
- No time-limited actions (except optional FTP transfer pacing)
- No reflex requirements
- `help` command always available
- `linux --help` provides full walkthrough for stuck players
- Font size controlled by player's terminal settings

---

## 10. Progression Model (Cross-Act)

### Prologue: The Upload
- **Unlock**: Basic Unix commands, FTP
- **Reward**: Linux is uploaded. History is made.
- **Carry forward**: QuestState, anomaly count, time played

### Act 1: (TBD)
- **Unlock**: New commands, new network hosts, new applications
- **Theme**: The machine begins to change. New files appear.
  The tty2 user leaves traces. Mail from unknown senders.

### Act 2: (TBD)
- **Unlock**: Elevated privileges, new areas of the filesystem
- **Theme**: The anomalies deepen. The OS itself behaves strangely.

Each act is a new scene in the engine, but the sidecar state carries
over. The player's filesystem, mail, history, and quest state persist.
The machine remembers everything.
