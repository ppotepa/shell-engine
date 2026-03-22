# Shell Quest — Script: Prologue

## Narrative Framework

### Tone

Not horror. Not comedy. **Quiet wrongness.**

The machine works. The commands return expected output. The mail is helpful.
But if you look carefully, some things don't add up. The dates, the users,
the network. Something knows more than it should.

The player is never told this. There are no cutscenes, no narration boxes,
no NPCs explaining the mystery. Everything is discovered through the
terminal. Read a log. Ping a host. Check who's logged in. The player
connects the dots themselves — or doesn't, and just finishes the upload.

### Time

**September 17, 1991, 21:12 UTC.**

This is the exact date Linux 0.01 was uploaded to nic.funet.fi. The game
clock starts at this moment. All timestamps in logs, mail headers, file
dates, and system output are consistent with this date.

Time advances in real-time (1 second = 1 second). If the player logs out
and returns 8 hours later, the system shows the delta. Mail accumulated.
Logs grew. Uptime increased. The anonymous user on tty2 is still there.

### Voice

The system speaks in lowercase. Terse. Unix culture:

```
you made it in. good.
read the notes when you get a chance.
```

```
remote: warning: linux-0.01.tar.Z - uncompress failed, archive may be damaged
remote: hint: check transfer mode (ascii vs binary)
```

No exclamation marks. No "Congratulations!". When the upload succeeds:

```
remote: linux-0.01.tar.Z received OK, archive integrity verified.
```

That's it. The machine doesn't celebrate. It confirms.

---

## Scene Flow

### Act 0 — Boot (scene: 05-intro-cpu-on)

The screen is black. Then:

```
MINIX 1.1 boot
```

Hardware detection lines appear one by one, each from the real kernel
probing actual hardware (values from MachineSpec based on difficulty):

```
memory: 4096K total, 109K kernel, 3987K free
hd driver: winchester, 20480K
clock: 100 Hz tick
tty: 3 virtual consoles
ethernet: 3Com EtherLink II at 0x300, IRQ 9    [OK]
```

Then the filesystem mounts and services start:

```
root filesystem: /dev/hd1 (minix)              [OK]
/usr filesystem: /dev/hd2 (minix)              [OK]
init: starting /etc/rc
starting netd...                               [OK]
starting maild...                              [OK]
starting cron...                               [OK]
```

Brief pause. Screen clears. Login prompt.

**Styling**: "MINIX" and version bright white. "OK" markers green.
Hardware model names highlighted. Everything else dim terminal green.

### Act 0.5 — Login (scene: 06-intro-login)

```
Minix 1.3  Copyright 1987, Prentice-Hall
Console ready

kruuna login: _
```

First time: player must type `linus`. Any other name:
```
first boot: login as linus
```

Then password (max 5 characters, masked with `*`). Account created.

Returning players: normal login flow, shows last login time.

### Act 1 — The Shell

After login:

```
account created.
last login: Tue Sep 17 21:12
you have 2 new messages.
type ls to look around.
type cat <file> to read notes.

linus@kruuna:~ [0]$ _
```

The player is now in a fully simulated MINIX shell. There is no
tutorial overlay, no quest tracker, no minimap. Just the prompt.

**The player is expected to:**

1. `ls` → see linux-0.01/, mail/, notes/
2. `cat notes/starter.txt` → basic command cheatsheet
3. `cat mail/welcome.txt` → "read the notes"
4. `cat mail/ast.txt` → **key hint**: Tanenbaum warns about binary mode
5. Optionally explore: `cat .bash_history` → see previous failed FTP attempt
6. Navigate to linux-0.01/: `cd linux-0.01`, `ls`, `cat README`
7. README contains explicit FTP instructions including `binary` command

**Hint escalation ladder** (from subtle to obvious):

| Hint Level | Source | Content |
|------------|--------|---------|
| 1 (subtle) | `.bash_history` | Shows `ascii` → `put` → `quit` sequence (no binary) |
| 2 (moderate) | `mail/ast.txt` | Tanenbaum: "compressed archives must transfer in binary mode" |
| 3 (direct) | `linux-0.01/README` | Explicit: step-by-step FTP instructions with `binary` |
| 4 (explicit) | `man ftp` | Manual page explains ASCII vs binary modes |
| 5 (cheat) | `linux --help` | Full walkthrough of the entire quest |

### Act 2 — The Upload

```
linus@kruuna:~/linux-0.01 [0]$ ftp nic.funet.fi
Connected to nic.funet.fi (128.214.6.100).
220 nic.funet.fi FTP server ready.
Name (nic.funet.fi:anonymous): anonymous
331 Guest login ok, send ident as password.
230 Guest login ok, access restrictions apply.
Remote system type is UNIX.
Using ascii mode to transfer files.

ftp nic.funet.fi> _
```

**The trap**: Default mode is ASCII (historically accurate). If the player
types `put linux-0.01.tar.Z` now, the transfer "succeeds" but:

```
226 Transfer complete.
73091 bytes sent in 2.4 seconds.

remote: warning: linux-0.01.tar.Z - uncompress failed, archive may be damaged
remote: hint: check transfer mode (ascii vs binary)
```

The hint clarity varies by difficulty (SU mode: just "transfer failed").

**The solution**: Type `binary` before `put`:

```
ftp nic.funet.fi> binary
200 Type set to I (binary).
ftp nic.funet.fi> put linux-0.01.tar.Z
200 PORT command successful.
150 Opening BINARY mode data connection for linux-0.01.tar.Z.
226 Transfer complete.
73091 bytes sent in 2.4 seconds.

remote: linux-0.01.tar.Z received OK, archive integrity verified.
```

The player can verify with `ls`:
```
ftp nic.funet.fi> ls
total 234
drwxr-xr-x  2 ftp  ftp  512 Sep 17 21:12 .
-rw-r--r--  1 ftp  ftp  73091 Sep 17 21:12 linux-0.01.tar.Z
```

`bye` exits FTP → back to shell → prologue complete → scene transition.

---

## The Mystery Thread (Optional — Parallel to Main Quest)

### Layer 1: Network anomalies

Player tries pinging hosts that shouldn't exist yet:

**google.com** (doesn't exist until 1998):
```
PING google.com: 56 data bytes
ping: sendto: Network is unreachable
--- google.com ---
request timed out (host unreachable)
note: destination 216.58.x.x — IANA block not yet allocated
```

**github.com** (doesn't exist until 2008):
```
PING github.com: 56 data bytes
From 128.214.1.1: Destination Host Unknown
route: partial trace — 4 hops then silence
note: 140.82.x.x — allocation not found in current IANA registry
```

**en.wikipedia.org** (doesn't exist until 2001):
```
PING en.wikipedia.org: 56 data bytes
request timed out
route analysis: destination 208.80.x.x responds to probe but
  returns no ICMP — routing table inconsistency
note: temporal anomaly in route path (this shouldn't happen)
```

Each anomaly creates/grows `/var/log/net.trace`:
```
[warn] 3 unresolvable hosts returned partial route data
[warn] destination networks not yet allocated by IANA
[warn] temporal inconsistency detected in routing tables
[    ] ...this shouldn't happen.
```

### Layer 2: System anomalies (after 3 network anomalies)

- `dmesg` adds: `[????] process 0: unnamed: started`
- `netstat` shows: `???    0.0.0.0:??    *:*    UNKNOWN`
- `date` occasionally glitches (shows future dates for a frame)
- `fortune` starts returning unsettling quotes

### Layer 3: The tty2 user

`who` output always shows:
```
linus    tty0    Sep 17 21:12
ast      tty1    Sep 15 09:41
(unknown) tty2   Jan  1 00:00
```

The tty2 user:
- Has no real name (unknown)
- Logged in at epoch zero (Jan 1 00:00:00 — the Unix timestamp origin)
- `finger tty2` returns: `Login: ???  Name: (no name)  Plan: (none — or is there?)`
- Appears in `/var/log/auth.log` with the impossible timestamp
- After prologue completion, disappears from `who` output

This user is never explained in the prologue. It's setup for Act 2.

### Layer 4: Tanenbaum's files

`/usr/ast/` contains Tanenbaum's working directory:
- `README` — "MINIX is for teaching, not production"
- `.plan` — working on MINIX 2.0
- `minix-2.0-notes.txt` — design notes mentioning "benchmark vs SunOS 4.1"

These are historically accurate. Tanenbaum really was working on MINIX 2.0
in late 1991. The files serve both as world-building and as subtle foreshadowing
of the Tanenbaum-Torvalds debate that would erupt in January 1992.

---

## Dialogue / System Messages — Full Script

### Boot messages
*(see Act 0 above — generated from MachineSpec, not hardcoded)*

### Login messages

**First login:**
```
account created.
last login: Tue Sep 17 21:12
you have 2 new messages.
type ls to look around.
type cat <file> to read notes.
```

**Returning login:**
```
last login: <previous login timestamp>
you have N new message(s).
type ls to look around.
type cat <file> to read notes.
```

### Mail: welcome.txt
```
From: Operator <op@kruuna>
Subject: Welcome

you made it in. good.
read the notes when you get a chance.
```

### Mail: ast.txt
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

### FTP upload fail (ASCII mode)
```
226 Transfer complete.
73091 bytes sent in X.X seconds.

remote: warning: linux-0.01.tar.Z - uncompress failed, archive may be damaged
remote: hint: check transfer mode (ascii vs binary)
```

### FTP upload success (binary mode)
```
226 Transfer complete.
73091 bytes sent in X.X seconds.

remote: linux-0.01.tar.Z received OK, archive integrity verified.
```

### Easter egg: linux --help
```
you found it. here's what to do:

1. the archive is in ~/linux-0.01/linux-0.01.tar.Z
2. connect: ftp nic.funet.fi
3. navigate: cd /pub/OS/Linux
4. IMPORTANT: type 'binary' (ascii corrupts .tar.Z files)
5. upload: put linux-0.01.tar.Z
6. verify: ls
7. done: bye

history was made with a 5-letter command.
```

### Easter egg: minix (3rd try)
```
I know.
```

---

## Scene Transitions

| From | Trigger | To |
|------|---------|-----|
| 04-difficulty-select | Player picks difficulty | 05-intro-cpu-on |
| 05-intro-cpu-on | Boot sequence complete | 06-intro-login |
| 06-intro-login | Successful FTP upload (binary) | Act 1 scene (TBD) |

The transition after successful upload is the end of the prologue.
The sidecar signals completion through quest state; the engine
reads it and triggers the scene change.
