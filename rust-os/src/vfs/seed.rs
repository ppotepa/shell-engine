use super::Vfs;

pub fn seed(vfs: &mut Vfs) {
    // --- Root directories ---
    for dir in &["/", "/bin", "/usr", "/usr/bin", "/usr/lib", "/usr/include",
                 "/etc", "/tmp", "/dev", "/proc", "/var", "/var/log",
                 "/var/spool", "/var/spool/mail", "/usr/adm",
                 "/usr/torvalds", "/usr/ast", "/usr/games",
                 "/usr/torvalds/linux-0.01", "/usr/torvalds/mail",
                 "/usr/torvalds/notes", "/usr/src", "/usr/src/minix",
                 "/usr/src/minix/kernel"] {
        vfs.mkdir(dir);
    }

    // --- /etc ---
    vfs.write_file("/etc/hostname", "kruuna", "root");
    vfs.write_file("/etc/motd",
        "MINIX 1.1  kruuna.helsinki.fi\n\
         System load: 0.12  Users logged in: 2\n\
         Helsinki University of Technology — Computer Science Lab\n",
        "root");
    vfs.write_file("/etc/passwd",
        "root:x:0:0:System Administrator:/:/bin/sh\n\
         daemon:x:1:1:Daemon:/:/bin/false\n\
         bin:x:2:2:Binary:/bin:/bin/false\n\
         ast:x:100:10:Andrew S. Tanenbaum:/usr/ast:/bin/sh\n\
         torvalds:x:1000:10:Linus Torvalds:/usr/torvalds:/bin/sh\n\
         nobody:x:65534:65534:Nobody:/:/bin/false\n",
        "root");
    vfs.write_file("/etc/group",
        "root:x:0:\n\
         daemon:x:1:\n\
         bin:x:2:\n\
         staff:x:10:ast,torvalds\n\
         wheel:x:11:torvalds\n\
         operator:x:12:ast\n\
         users:x:100:ast,torvalds\n",
        "root");
    vfs.write_file("/etc/hosts",
        "127.0.0.1       localhost kruuna kruuna.helsinki.fi\n\
         128.214.6.100   nic.funet.fi ftp.funet.fi\n\
         130.37.24.3     cs.vu.nl\n\
         128.214.3.1     helsinki.fi\n\
         128.214.22.1    tut.fi\n\
         130.188.1.1     oulu.fi\n",
        "root");
    vfs.write_file("/etc/services",
        "ftp-data    20/tcp\n\
         ftp         21/tcp\n\
         telnet      23/tcp\n\
         smtp        25/tcp\n\
         finger      79/tcp\n\
         http        80/tcp\n",
        "root");
    vfs.write_file("/etc/rc",
        "#!/bin/sh\n\
         # /etc/rc — system initialization\n\
         /etc/update &\n\
         /etc/cron &\n\
         echo 'System services started.'\n",
        "root");
    vfs.write_file_with_perms("/etc/rc",
        "#!/bin/sh\n\
         # /etc/rc — system initialization\n\
         /etc/update &\n\
         /etc/cron &\n\
         echo 'System services started.'\n",
        "root", "-rwxr-xr-x");

    // --- /usr/torvalds home ---
    vfs.write_file("/usr/torvalds/.profile",
        "# .profile\n\
         PATH=/bin:/usr/bin\n\
         TERM=minix\n\
         export PATH TERM\n\
         umask 022\n",
        "torvalds");
    vfs.write_file("/usr/torvalds/.plan",
        "working on something. will post to comp.os.minix.\n",
        "torvalds");
    vfs.write_file("/usr/torvalds/.sh_history",
        "ls\n\
         ls -la\n\
         ftp nic.funet.fi\n\
         ascii\n\
         put linux-0.01.tar.Z\n\
         quit\n",
        "torvalds");

    // --- linux-0.01 source ---
    vfs.write_file("/usr/torvalds/linux-0.01/README",
        "INSTALL notes:\n\
         \n\
         This is a free minix-like kernel for 386/486 AT-machines.\n\
         It is currently 'as is', but I intend to make it somewhat more\n\
         portable, better-documented, supported etc.\n\
         \n\
         Linux kernel source v0.01 — L. Torvalds, September 1991\n\
         \n\
         To upload to nic.funet.fi:\n\
         - Connect via FTP (modem required)\n\
         - Switch to BINARY mode before transfer\n\
         - put linux-0.01.tar.Z\n\
         \n\
         WARNING: ascii mode will corrupt compressed archives.\n",
        "torvalds");
    vfs.write_file("/usr/torvalds/linux-0.01/RELNOTES-0.01",
        "Release notes for Linux 0.01 — September 17, 1991\n\
         \n\
         Linux is a MINIX-like clone for 386 machines.\n\
         It uses 386 task switching for multitasking.\n\
         VFS based on MINIX FS design by Prof. A. Tanenbaum.\n\
         \n\
         Kernel size: approx 64K\n\
         Requires: 386 or better, 4MB RAM minimum\n",
        "torvalds");
    vfs.write_file_with_perms(
        "/usr/torvalds/linux-0.01/linux-0.01.tar.Z",
        "[COMPRESSED ARCHIVE — 73091 bytes — binary data]\n",
        "torvalds", "-rw-r--r--");
    vfs.write_file_with_perms(
        "/usr/torvalds/linux-0.01/bash.Z",
        "[COMPRESSED ARCHIVE — bash 1.05 — binary data]\n",
        "torvalds", "-rw-r--r--");
    vfs.write_file_with_perms(
        "/usr/torvalds/linux-0.01/update.Z",
        "[COMPRESSED ARCHIVE — update daemon — binary data]\n",
        "torvalds", "-rw-r--r--");

    // --- mail ---
    vfs.write_file("/usr/torvalds/mail/welcome.txt",
        "From: op@kruuna\n\
         To: torvalds@kruuna\n\
         Subject: welcome\n\
         Date: Mon, 16 Sep 1991 18:42:00 +0300\n\
         \n\
         you made it in. good.\n\
         \n\
         the system is running minix 1.1.\n\
         accounts are limited. use resources wisely.\n\
         \n\
         — op\n",
        "torvalds");
    vfs.write_file("/usr/torvalds/mail/ast.txt",
        "From: ast@cs.vu.nl\n\
         To: torvalds@kruuna\n\
         Subject: Re: your kernel\n\
         Date: Mon, 16 Sep 1991 20:11:00 +0200\n\
         \n\
         Linus,\n\
         \n\
         Interesting project. I looked at the code.\n\
         One note on the file transfer: compressed archives (.Z files)\n\
         must be transferred in BINARY mode. ASCII mode corrupts the\n\
         compression headers. This is not negotiable.\n\
         \n\
         — ast\n",
        "ast");

    // --- notes ---
    vfs.write_file("/usr/torvalds/notes/starter.txt",
        "Notes to self:\n\
         - try 'man ftp' for upload instructions\n\
         - type 'ps' to see running processes\n\
         - type 'ls -la' to see hidden files\n\
         - ping nic.funet.fi to test the network connection\n",
        "torvalds");

    // --- /usr/ast ---
    vfs.write_file("/usr/ast/.plan",
        "Working on MINIX 2.0 design.\n\
         MINIX is and will remain a teaching tool.\n\
         Commercial use is not the intent.\n\
         — ast, VU Amsterdam\n",
        "ast");
    vfs.write_file("/usr/ast/minix-philosophy.txt",
        "MINIX was designed as a teaching OS.\n\
         The source is available precisely because students should read it.\n\
         A kernel should be small enough to understand.\n\
         Linux is too large. It will never be maintainable.\n\
         — A. S. Tanenbaum\n",
        "ast");

    // --- /tmp detritus ---
    vfs.write_file("/tmp/thesis-FINAL-v3-REAL.bak",
        "[binary backup — unknown format]\n",
        "torvalds");
    vfs.write_file("/tmp/core",
        "[core dump — cc1 segmentation fault — 4096 bytes]\n",
        "root");
    vfs.write_file("/tmp/nroff-err.log",
        "nroff: cannot find font R\n\
         nroff: cannot find font B\n\
         nroff: 14 formatting errors\n",
        "root");
    vfs.write_file("/tmp/.Xauthority",
        "[binary Xauth data]\n",
        "torvalds");
    vfs.write_file("/tmp/.lock-ast",
        "ast:tty1:1991-09-17\n",
        "ast");

    // Hidden note only visible with ls -a /tmp
    vfs.write_file("/tmp/.hidden_note",
        "it compiled. first try. I don't believe it.\n\
         tomorrow I post to comp.os.minix.\n\
         —L\n",
        "torvalds");

    // --- /var/log ---
    vfs.write_file("/var/log/messages",
        "Sep 17 21:12:00 kernel: MINIX 1.1 (i386)\n\
         Sep 17 21:12:00 kernel: memory: 4096K total, 3584K available\n\
         Sep 17 21:12:01 hd1: Seagate ST-157A, 40960K\n\
         Sep 17 21:12:01 eth0: NE2000 compatible at 0x300\n\
         Sep 17 21:12:02 tty0: getty started\n\
         Sep 17 21:12:09 login: torvalds logged in on tty0\n",
        "root");
    vfs.write_file("/var/log/cron",
        "Sep 17 21:12:05 cron: started\n\
         Sep 17 21:15:00 cron: running /etc/cron.15min\n",
        "root");
    vfs.write_file_with_perms("/var/log/wtmp",
        "[binary login accounting data]\n",
        "root", "-rw-r-----");

    // --- /usr/adm ---
    vfs.write_file("/usr/adm/messages",
        "Sep 17 21:12:09 login: torvalds logged in on tty0\n\
         Sep 17 21:12:09 login: ast logged in on tty1\n",
        "root");

    // --- /usr/src/minix/kernel ---
    vfs.write_file("/usr/src/minix/kernel/main.c",
        "/* MINIX 1.1 — A. Tanenbaum, 1987 */\n\
         /* \"MINIX is for teaching, not production\" */\n\
         \n\
         /* NOTE: someone has been modifying the scheduler. */\n\
         /* The changes don't match any known patch. —ast    */\n\
         \n\
         #include <minix/config.h>\n\
         #include <minix/type.h>\n\
         \n\
         /* main() — kernel entry point */\n\
         void main() {\n\
             sys_init();\n\
             mem_init();\n\
             clock_init();\n\
             /* ... */\n\
         }\n",
        "root");

    // --- /usr/games ---
    vfs.write_file("/usr/games/fortune.dat",
        "[binary fortune database]\x00\x01\x03\
         the future is already here, it's just not evenly distributed\
         \x00\x02\x01[end]\n",
        "root");

    // --- /dev entries (virtual) ---
    vfs.write_file("/dev/null", "", "root");
    vfs.write_file("/dev/console", "[console device]", "root");
    vfs.write_file("/dev/tty0", "[terminal device tty0]", "root");
    vfs.write_file("/dev/tty1", "[terminal device tty1]", "root");
    vfs.write_file("/dev/tty2", "[terminal device tty2]", "root");
    vfs.write_file("/dev/hd1", "[hard disk partition 1]", "root");
    vfs.write_file("/dev/hd2", "[hard disk partition 2]", "root");

    // --- /proc ---
    vfs.write_file("/proc/meminfo",
        "MemTotal:       4096 kB\n\
         MemFree:        2891 kB\n\
         Buffers:         512 kB\n",
        "root");

    // --- /var/spool/mail ---
    vfs.write_file("/var/spool/mail/torvalds",
        "[mail spool — 2 messages]\n",
        "torvalds");
}
