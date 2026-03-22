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

| Type | Behavior |
|------|----------|
| `Normal` | Reachable, responds to ping, may host FTP |
| `PingOnly` | Responds to ping but no services |
| `Anomaly` | Temporal anomaly — spooky error sequence |
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

| Host | IP | Base Latency | Note |
|------|----|--------------|------|
| `nic.funet.fi` | `128.214.6.100` | 47ms | Quest target |
| `ftp.funet.fi` | `128.214.6.100` | 47ms | Alias → same server |
| `cs.vu.nl` | `130.37.24.3` | 112ms | Tanenbaum's university |
| `sun.com` | `192.9.9.1` | 203ms | Sun Microsystems |
| `helsinki.fi` | `128.214.1.1` | 12ms | Local university |
| `ftp.uu.net` | `192.48.96.9` | 189ms | UUNET archives |
| `localhost` | `127.0.0.1` | 0.01ms | Loopback |
| `kruuna` | `127.0.0.1` | 0.01ms | Self |

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

**Key detail:** these are NOT `unknown host`. The system *tries* to resolve them
and *partially succeeds*. That's the unsettling part.

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

| Input | Response |
|-------|----------|
| `emacs` | `emacs: not installed. only vi available on this system.` |
| `vi` | *(200ms delay)* `vi: insufficient memory` |
| `rm` (anything) | `rm: permission denied (nice try)` |
| `su` | `su: incorrect password` |
| `su` (if difficulty=SU) | `su: you chose this name, didn't you?` |
| `shutdown` | `shutdown: must be superuser.` |
| `make` | `make: no targets. nothing to do.` |
| `gcc` | `gcc: not installed. try Amsterdam Compiler Kit.` |
| `reboot` | `reboot: must be superuser.` |
| `finger ast` | Shows Tanenbaum's `.plan` file (see C4) |
| `finger linus` | `Login: linus  Name: Linus Torvalds  Home: /home/linus` |
| `finger tty2` | `finger: tty2: no such user.` |
| `echo hello` | `hello` |
| `cat /dev/null` | *(no output — correct behavior)* |
| `cat /dev/random` | `[random bytes — display corrupted]` then garbled chars |

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

| Level | Source | Directness | How player finds it |
|-------|--------|-----------|---------------------|
| 1 | `.bash_history` | Indirect | `cat .bash_history` — see failed ascii attempt |
| 2 | `mail/ast.txt` | Moderate | "2 new messages" prompt → `cat mail/ast.txt` |
| 3 | `linux-0.01/README` | Moderate | `cd linux-0.01 && cat README` — upload instructions |
| 4 | `man ftp` | Direct | Curious player checks manual — TRANSFER MODES section |
| 5 | `linux --help` | Explicit | Desperate player types `linux --help` — full walkthrough |

Each level is reachable through natural curiosity. No hint is forced.

---

## F) Implementation Priority

Ordered by atmosphere-per-effort ratio:

| Priority | Feature | Effort | Impact |
|----------|---------|--------|--------|
| 1 | `IExternalServer` + `NetworkRegistry` | Medium | Foundation for ping + future network |
| 2 | `PingCommand` + anomalies | Medium | Strongest unique feature |
| 3 | `IEasterEgg` registry + all one-liners | Low | High charm, trivial code |
| 4 | `minix` (silent/stateful) + `linux --help` | Low | Memorable moments |
| 5 | `.bash_history` + `mail/ast.txt` | Low | Quest hint infrastructure |
| 6 | `man ftp` | Low | Stuck-player safety net |
| 7 | `/etc/motd` | Low | Login polish |
| 8 | `fortune`, `uptime`, `date`, `uname`, `whoami`, `who` | Low | Env commands batch |
| 9 | Filesystem extras (passwd, tmp, ast, /proc/version) | Low | Exploration rewards |
| 10 | `dmesg` + `/var/log/messages` | Low | Deep lore for thorough players |
| 11 | Disk seek flavor | Low | Nice-to-have polish |
| 12 | `date` glitch + `/proc/version` glitch | Low | Post-anomaly spooky payoff |

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
