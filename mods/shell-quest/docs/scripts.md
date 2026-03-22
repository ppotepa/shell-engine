# shell-quest — scripts

design doc for quests, story beats, and scene flow.
all text here is draft / wip. simple language on purpose.

---

## difficulty select (04-difficulty-select)

### what it is

player picks difficulty. five options shown as 3d portraits with labels.
scene has `prerender: true` — all OBJ frames are baked synchronously before
scene activates (no loading screen needed).

### portraits

each difficulty has a wire + solid sprite pair. on selection the
`portrait-materialize` behavior runs a wireframe→solid scanline reveal:

- both spin from 180° to 0° in 250ms
- first 90ms: glitch phase (blink between wireframe and material)
- then scanline sweeps top→bottom: solid appears above, wire below
- ends with solid facing camera at rotation-y 220°

all 5 portraits use identical animation params (`dur: 250`, `rotation-y: 220`)
so every selection looks and feels the same.

### difficulty options

| key | label | codename |
|-----|-------|----------|
| 1 | MOUSE ENJOYER | easy |
| 2 | SCRIPT KIDDIE | normal-easy |
| 3 | I CAN EXIT VIM | normal |
| 4 | DVORAK | hard |
| 5 | SU | nightmare |

all route to `05.intro.cpu-on` (boot sequence). difficulty affects
MachineSpec (cpu, ram, nic speed) — see prologue section below.

---

## prologue — the upload (september 1991)

### setting

player is linus torvalds. year 1991. helsinki university.
machine runs minix 1.3 on a 386/486 (depends on difficulty).
goal: upload early linux kernel sources to nic.funet.fi via ftp.

### the puzzle

ftp defaults to ascii transfer mode (historically accurate).
ascii mode corrupts binary archives (.tar.Z).
player must figure out that `binary` command fixes the transfer.

### scene flow

```
boot sequence (05-intro-cpu-on)
  → hardware detection from MachineSpec
  → minix banner
  → login prompt (06-intro-login)

login as linus
  → shell prompt: linus@kruuna:/home/linus $

explore filesystem
  → /home/linus/linux-0.01/ has the source files
  → key file: linux-0.01.tar.Z (the archive to upload)
  → also: RELNOTES-0.01, bash.Z, update.Z

optional: cp linux-0.01.tar.Z linux-0.01.tar.Z.bak
  → safety copy before upload attempt

ftp nic.funet.fi
  → connects, anonymous login
  → default mode: ascii (shown on connect)
  → cd /pub/OS/Linux

put linux-0.01.tar.Z
  → transfer "succeeds" but...
  → remote warning: archive damaged / can't uncompress
  → hint: check transfer mode

player discovers the problem
  → types: binary
  → retries: put linux-0.01.tar.Z
  → remote confirms: archive OK

bye → back to shell
  → quest complete message or scene transition
```

### difficulty impact

| aspect | easy (mouse enjoyer) | normal (i can exit vim) | hard (su) |
|--------|---------------------|------------------------|-----------|
| cpu | 486 DX2-66 | 386 DX-33 | 386 SX-16 |
| ram | 8192 KB | 4096 KB | 1024 KB |
| nic speed | 10000 Kbps | 2400 Kbps | 1200 Kbps |
| transfer time | fast | medium | slow |
| hints | remote gives clear hint | remote gives vague hint | no hint, just "failed" |

### key commands player needs

- `ls` — list files
- `cat RELNOTES-0.01` — read release notes (optional lore)
- `cp` — backup files (optional but smart)
- `cd` — navigate directories
- `ftp <host>` — enter ftp client
- `open`, `binary`, `ascii`, `put`, `ls`, `bye` — ftp commands

### notes

- all of this runs inside the c# sidecar (cognitOS)
- engine just shows what sidecar sends via ipc
- no real network — everything simulated in FtpSession.cs
- quest state tracked in QuestState (Models.cs)
- MachineSpec controls all hardware numbers

---

## act 1 — (tbd)

placeholder for next story arc after prologue.
