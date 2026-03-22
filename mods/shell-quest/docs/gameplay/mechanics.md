# Shell Quest — Gameplay Mechanics

## 1. Input Model

The player types into a single-line input field. Every `Enter` sends
the line to the sidecar as a `submit` message. The sidecar processes
it, updates state, and sends back a `screen-full` frame.

There is no mouse interaction. No clickable UI. No menus (within the OS).
Everything is keyboard-driven, exactly like a real 1991 terminal.

**Key events the engine sends:**
- `hello` — initial handshake (difficulty, viewport size, boot_scene flag)
- `tick` — every frame (~16ms), with dt_ms
- `resize` — terminal viewport changed
- `set-input` — keystrokes as the player types (before Enter)
- `submit` — player pressed Enter

**Responses from sidecar:**
- `screen-full` — complete terminal frame (lines + cursor position)
- `set-prompt-prefix` — prompt text (linus@kruuna:~ $)
- `set-prompt-masked` — password mode (show * instead of chars)
- `out` — append lines to screen (legacy, mostly screen-full now)

### Input display

- Prompt is rendered in color: `user@host:path [exitcode]$`
- Player's typed text is rendered 10% dimmer (#adadad) for style
- Password input shows `*` characters, actual text hidden
- Empty submit (just Enter) is ignored

---

## 2. Session Lifecycle

```
BOOT → LOGIN → SHELL → [APPLICATIONS] → SHELL → ...
         ↑                                    │
         └────────────── logout ──────────────┘
```

### Boot phase
- Mode: `Booting`
- Input disabled (prompt empty)
- Boot steps play as timed text lines
- Each line appears after delay (simulating hardware init)
- After last step: 500ms pause → clear → login prompt

### Login phase
- Mode: `LoginUser` → `LoginPassword`
- First boot: only `linus` accepted, creates account
- Return visit: checks username + password
- Failed login: "login incorrect", back to username
- Success: `EnterShell()`

### Shell phase
- Mode: `Shell`
- `ShellApplication` is bottom of ApplicationStack
- Commands dispatched: CommandIndex → EasterEggs → "not found"
- Some commands launch applications (FTP) → pushed onto stack
- Stack input always routes to topmost application
- When app exits (bye/quit), popped → shell regains focus

### Application stack
```
┌─────────────────┐
│  FtpApplication  │  ← topmost: gets all input
├─────────────────┤
│ ShellApplication │  ← waiting below
└─────────────────┘
```

Each application has:
- `PromptPrefix()` — what the prompt shows
- `OnEnter()` — setup on push
- `OnExit()` — cleanup on pop
- `HandleInput()` → `Continue` or `Exit`

---

## 3. Command Execution

### Current flow
```
player types "ping sun.nl" → Enter
  ↓
ShellApplication.HandleInput("ping sun.nl")
  ↓
parse: cmd="ping", argv=["sun.nl"]
  ↓
CommandIndex["ping"] → PingCommand
  ↓
PingCommand.Execute(ctx) → CommandResult(lines, exitCode)
  ↓
ShellApplication renders lines to ScreenBuffer
  ↓
ScreenBuffer sends screen-full frame to engine
```

### CommandContext — what commands receive
```csharp
record CommandContext(
    IOperatingSystem Os,    // full OS access
    UserSession Session,    // cwd, user, hostname, exit code
    string CommandName,     // "ping"
    IReadOnlyList<string> Argv  // ["sun.nl"]
)
```

### CommandResult — what commands return
```csharp
record CommandResult(
    IReadOnlyList<string> Lines,  // output text (markup-enabled)
    int ExitCode = 0,
    bool ClearScreen = false,
    string? LaunchApp = null       // "ftp" → push FtpApplication
)
```

### Exit codes
- 0 = success
- 1 = general error
- 127 = command not found
- Displayed in prompt: `[0]$` or `[1]$`

---

## 4. Command Dispatch Order

When the player types a command:

```
1. Check CommandIndex (registered ICommand implementations)
   → Found? Execute it.

2. Check EasterEggRegistry
   → Match? Return special response.

3. Neither? → "command not found" (exit code 127)
```

Easter eggs are checked AFTER real commands so they can't shadow
legitimate functionality. They handle things like `minix`, `linux`,
`emacs`, `vi` — words that aren't real commands but have special meaning.

---

## 5. Filesystem Interaction

### Virtual File System (VFS)

All files live in memory. Persisted to `state.obj` (ZIP archive).
On every reload, epoch files are re-seeded (they're not in the ZIP).

**Operations available to commands:**
- `Ls(path)` — list directory entries
- `TryCat(path, out content)` — read file
- `TryCopy(src, dst, out error)` — copy file
- `TryWrite(path, content, out error)` — write file
- `TryMkdir(path, out error)` — create directory
- `DirectoryExists(path)` — check directory
- `ToVfsPath(absolutePath)` — convert /home/linus/foo → foo

**Path resolution:**
- `UserSession.ResolvePath()` handles ~, .., ., relative, absolute
- VFS paths are relative to /home/linus (stripped prefix)
- System paths (/etc, /var) stored with path minus leading /

### What the player can modify
- `cp` — copy files
- FTP `put` — simulated upload (modifies QuestState, not real VFS)
- Future: `echo "text" > file`, `mkdir`, `rm`

### What's read-only (seeded)
- Everything in /etc, /usr, /tmp, /var, /proc, /dev
- mail/ (content seeded, read-tracking via MailMessage state)
- linux-0.01/ (the source archive)

---

## 6. Quest State Tracking

Quest progress is tracked in `QuestState` (persisted in state.obj):

```csharp
class QuestState
{
    string FtpTransferMode = "ascii";   // current FTP mode
    bool UploadAttempted;               // tried to put
    bool BackupMade;                    // cp'd the archive
    bool UploadSuccess;                 // binary + put = success
    string? FtpRemoteHost;             // connected host
    bool FtpConnected;                  // active FTP session
    List<string>? AnomaliesDiscovered;  // pinged temporal hosts
}
```

**Quest completion condition:** `UploadSuccess == true`

The engine reads this from the sidecar state (via IPC or state file)
to trigger the scene transition to Act 1.

---

## 7. Difficulty Impact on Gameplay

Difficulty doesn't change what commands are available. It changes
the **texture** of the experience:

### Hardware differences
- **CPU model** → shown in dmesg, uname, boot sequence
- **RAM amount** → shown in free, dmesg, top
- **NIC speed** → affects FTP transfer time simulation
- **Disk size** → shown in df

### Hint differences
- **Easy**: FTP upload failure gives explicit "check transfer mode (ascii vs binary)"
- **Normal**: "archive may be damaged" + vague "check transfer mode"
- **Hard**: Just "transfer failed" — no hint at all

### Pacing differences
- **Fast NIC** (10 Mbps): FTP transfer takes 0.1 seconds
- **Slow NIC** (300 bps): FTP transfer takes 30+ seconds (tense waiting)

---

## 8. Discovery Mechanics

The game has no quest log, no markers, no "!" above NPCs. Discovery
is entirely through **reading what the system tells you**.

### Information sources

| Source | How to access | What it reveals |
|--------|--------------|-----------------|
| `ls` | Type `ls` | Files in current directory |
| `cat <file>` | Read any file | Content — mail, notes, configs |
| `.bash_history` | `cat .bash_history` | Previous session (failed upload) |
| `mail/ast.txt` | `cat mail/ast.txt` | Binary mode hint from Tanenbaum |
| `man ftp` | `man ftp` | Full FTP manual including modes |
| `linux --help` | Easter egg | Complete walkthrough |
| `/var/log/*` | `cat /var/log/messages` | System logs (anomaly traces) |
| `who` | Type `who` | Logged-in users (tty2 mystery) |
| `finger` | `finger <user>` | User details |
| `dmesg` | After anomalies | Kernel anomalies |
| `ping` | Ping future hosts | Temporal anomalies |

### Reward for exploration
There are no XP points or achievements. The reward is **understanding**.
The player who reads everything knows:
- Why ASCII mode corrupts archives (real CS knowledge)
- Who Tanenbaum is and what MINIX is for
- That something strange is happening with the network
- That an unknown user is on tty2 at epoch zero

This knowledge carries into Act 2.

---

## 9. Screen Rendering

### Viewport
- Configurable columns × rows (default 120×40)
- Engine sends viewport size on hello and resize
- ScreenBuffer wraps lines that exceed column width
- Markup tags `[#color]text[/]` are zero-width (don't count for wrapping)

### Scrolling
- When content exceeds viewport rows, oldest lines scroll off top
- No scroll-back (authentic 1991 terminal behavior)
- `clear` command empties the viewport

### Colors (markup system)
- `[#rrggbb]text[/]` → colored text
- `Style.Fg(color, text)` helper wraps text in tags
- Engine-side parser renders actual terminal colors

**Color palette:**
- Prompt user: green
- Prompt host: cyan
- Prompt path: blue
- Errors: red
- Warnings: yellow
- Info: bright terminal green
- User input: #adadad (10% dimmer than default)
- Application output: +15% brightness vs. base

### Frame protocol
Every state change sends a complete `screen-full`:
```json
{
  "type": "screen-full",
  "cols": 120,
  "rows": 40,
  "lines": ["line1", "line2", ...],
  "cursor_x": 23,
  "cursor_y": 39
}
```
Engine replaces entire terminal content. No partial updates.

---

## 10. Persistence

### What's saved (state.obj ZIP)
- `manifest.json` — schema version, timestamps
- `users/linus/profile.json` — username, password, last login
- `users/linus/home/**` — all VFS files modified by player
- Machine state — uptime, processes, services, quest progress

### What's NOT saved (re-seeded every load)
- Epoch files (linux-0.01/, mail/, notes/, /etc, /usr, etc.)
- Default process table
- Default service list
- Default mail messages

### Save trigger
State is persisted on:
- Account creation (first login)
- Shell entry (every login)
- FTP upload attempt
- Future: any file modification

### First boot vs return
- First boot: state.obj doesn't exist → created with defaults
- Return: state.obj loaded → VFS populated from ZIP + epoch seed
