namespace CognitOS.Network.Hosts;

// ── Loopback ────────────────────────────────────────────────────────────────

[RemoteHost("localhost", Aliases = ["kruuna"])]
internal sealed class Localhost : IRemoteHost
{
    public string Hostname  => "localhost";
    public IReadOnlyList<string> Aliases => ["kruuna"];
    public string IpAddress => "127.0.0.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Loopback;
}

// ── Finnish academic network (FUNET) ────────────────────────────────────────

[RemoteHost("nic.funet.fi", Aliases = ["ftp.funet.fi"])]
internal sealed class NicFunetFi : IRemoteHost
{
    public string Hostname  => "nic.funet.fi";
    public IReadOnlyList<string> Aliases => ["ftp.funet.fi"];
    public string IpAddress => "128.214.6.100";
    public int BasePingMs   => 47;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("helsinki.fi")]
internal sealed class HelsinkiFi : IRemoteHost
{
    public string Hostname  => "helsinki.fi";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "128.214.1.1";
    public int BasePingMs   => 12;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("tut.fi")]
internal sealed class TutFi : IRemoteHost
{
    public string Hostname  => "tut.fi";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "130.230.1.1";
    public int BasePingMs   => 18;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("oulu.fi")]
internal sealed class OuluFi : IRemoteHost
{
    public string Hostname  => "oulu.fi";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "130.231.1.1";
    public int BasePingMs   => 22;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("utu.fi")]
internal sealed class UtuFi : IRemoteHost
{
    public string Hostname  => "utu.fi";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "130.232.1.1";
    public int BasePingMs   => 25;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("jyu.fi")]
internal sealed class JyuFi : IRemoteHost
{
    public string Hostname  => "jyu.fi";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "130.234.1.1";
    public int BasePingMs   => 38;
    public HostAccess Access => HostAccess.PingOnly;
}

// ── European academic sites ──────────────────────────────────────────────────

[RemoteHost("cs.vu.nl")]
internal sealed class CsVuNl : IRemoteHost  // Tanenbaum's machine
{
    public string Hostname  => "cs.vu.nl";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "130.37.24.3";
    public int BasePingMs   => 112;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("ethz.ch")]
internal sealed class EthzCh : IRemoteHost  // ETH Zurich — Wirth, Oberon, Pascal
{
    public string Hostname  => "ethz.ch";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "129.132.1.1";
    public int BasePingMs   => 127;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("doc.ic.ac.uk")]
internal sealed class DocIcAcUk : IRemoteHost  // Imperial College London
{
    public string Hostname  => "doc.ic.ac.uk";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "155.198.0.1";
    public int BasePingMs   => 134;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("ftp.informatik.tu-muenchen.de")]
internal sealed class FtpInformatikTuMuenchenDe : IRemoteHost
{
    public string Hostname  => "ftp.informatik.tu-muenchen.de";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "131.159.0.1";
    public int BasePingMs   => 119;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("ftp.ibp.fr")]
internal sealed class FtpIbpFr : IRemoteHost  // Institut Blaise Pascal, Paris
{
    public string Hostname  => "ftp.ibp.fr";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "132.227.60.2";
    public int BasePingMs   => 141;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("archive.cs.ruu.nl")]
internal sealed class ArchiveCsRuuNl : IRemoteHost  // Utrecht university archive
{
    public string Hostname  => "archive.cs.ruu.nl";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "131.211.0.1";
    public int BasePingMs   => 108;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("info.cern.ch")]
internal sealed class InfoCernCh : IRemoteHost  // Tim Berners-Lee's first web server
{
    public string Hostname  => "info.cern.ch";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "128.141.201.74";
    public int BasePingMs   => 148;
    public HostAccess Access => HostAccess.Normal;
}

// ── North American sites ─────────────────────────────────────────────────────

[RemoteHost("prep.ai.mit.edu", Aliases = ["ftp.gnu.org"])]
internal sealed class PrepAiMitEdu : IRemoteHost  // GNU FTP, first Linux mirror
{
    public string Hostname  => "prep.ai.mit.edu";
    public IReadOnlyList<string> Aliases => ["ftp.gnu.org"];
    public string IpAddress => "18.71.0.38";
    public int BasePingMs   => 231;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("wuarchive.wustl.edu")]
internal sealed class WuarchiveWustlEdu : IRemoteHost
{
    public string Hostname  => "wuarchive.wustl.edu";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "128.252.135.4";
    public int BasePingMs   => 244;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("gatekeeper.dec.com")]
internal sealed class GatekeeperDecCom : IRemoteHost  // DEC public archive
{
    public string Hostname  => "gatekeeper.dec.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "16.1.0.2";
    public int BasePingMs   => 219;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("research.att.com")]
internal sealed class ResearchAttCom : IRemoteHost  // Bell Labs
{
    public string Hostname  => "research.att.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "135.104.1.1";
    public int BasePingMs   => 207;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("cs.cmu.edu")]
internal sealed class CsCmuEdu : IRemoteHost  // Carnegie Mellon
{
    public string Hostname  => "cs.cmu.edu";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "128.2.0.90";
    public int BasePingMs   => 238;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("cs.berkeley.edu")]
internal sealed class CsBerkeleyEdu : IRemoteHost  // UC Berkeley
{
    public string Hostname  => "cs.berkeley.edu";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "128.32.0.1";
    public int BasePingMs   => 252;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("cs.stanford.edu")]
internal sealed class CsStanfordEdu : IRemoteHost
{
    public string Hostname  => "cs.stanford.edu";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "36.0.0.5";
    public int BasePingMs   => 248;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("mit.edu")]
internal sealed class MitEdu : IRemoteHost
{
    public string Hostname  => "mit.edu";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "18.72.2.1";
    public int BasePingMs   => 231;
    public HostAccess Access => HostAccess.PingOnly;
}

[RemoteHost("ftp.uu.net", Aliases = ["uunet.uu.net"])]
internal sealed class FtpUuNet : IRemoteHost
{
    public string Hostname  => "ftp.uu.net";
    public IReadOnlyList<string> Aliases => ["uunet.uu.net"];
    public string IpAddress => "192.48.96.9";
    public int BasePingMs   => 189;
    public HostAccess Access => HostAccess.Normal;
}

[RemoteHost("sun.com")]
internal sealed class SunCom : IRemoteHost
{
    public string Hostname  => "sun.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "192.9.9.1";
    public int BasePingMs   => 203;
    public HostAccess Access => HostAccess.PingOnly;
}

// ── Pacific / Oceania ────────────────────────────────────────────────────────

[RemoteHost("munnari.oz.au")]
internal sealed class MunnariOzAu : IRemoteHost  // Australian internet hub
{
    public string Hostname  => "munnari.oz.au";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "128.250.1.21";
    public int BasePingMs   => 351;
    public HostAccess Access => HostAccess.PingOnly;
}
