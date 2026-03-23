using CognitOS.Kernel;

namespace CognitOS.Network.Hosts;

// All hosts below implement IEasterEgg. Execute() writes all visible output directly
// to uow.Out. PingCommand handles quest tracking / net.trace update before calling Execute().

[RemoteHost("google.com")]
internal sealed class GoogleCom : IEasterEgg
{
    public string Hostname  => "google.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "216.58.209.14";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING google.com (216.58.209.14): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("Request timeout for icmp_seq 1");
        uow.Out.WriteLine("64 bytes from 216.58.209.14: icmp_seq=2 ttl=54 time=12ms");
        uow.Out.WriteLine("--- google.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 1 received, 66% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 12/12/12 ms");
        uow.Out.WriteLine("net: warning: 216.58.209.14 is not allocated by IANA");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("github.com")]
internal sealed class GithubCom : IEasterEgg
{
    public string Hostname  => "github.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "140.82.121.4";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING github.com (140.82.121.4): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 140.82.121.4: icmp_seq=0 ttl=47 time=8ms");
        uow.Out.WriteLine("64 bytes from 140.82.121.4: icmp_seq=1 ttl=47 time=7ms");
        uow.Out.WriteLine("64 bytes from 140.82.121.4: icmp_seq=2 ttl=47 time=9ms");
        uow.Out.WriteLine("--- github.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 7/8/9 ms");
        uow.Out.WriteLine("net: warning: 140.82.121.4 is not allocated by IANA");
        uow.Out.WriteLine("net: route fragment timestamp: 10 Apr 2008 00:00:01 UTC (clock skew detected)");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("wikipedia.org")]
internal sealed class WikipediaOrg : IEasterEgg
{
    public string Hostname  => "wikipedia.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "208.80.154.224";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING wikipedia.org (208.80.154.224): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("Request timeout for icmp_seq 1");
        uow.Out.WriteLine("Request timeout for icmp_seq 2");
        uow.Out.WriteLine("--- wikipedia.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 0 received, 100% packet loss");
        uow.Out.WriteLine("net: error: NXDOMAIN — but partial route fragment received from AS 14907");
        uow.Out.WriteLine("net: route fragment timestamp: 15 Jan 2001 00:13:01 UTC (inconsistent with local clock)");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("kernel.org")]
internal sealed class KernelOrg : IEasterEgg
{
    public string Hostname  => "kernel.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "198.145.20.140";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING kernel.org (198.145.20.140): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 198.145.20.140: icmp_seq=0 ttl=51 time=183ms");
        uow.Out.WriteLine("64 bytes from 198.145.20.140: icmp_seq=1 ttl=51 time=179ms");
        uow.Out.WriteLine("64 bytes from 198.145.20.140: icmp_seq=2 ttl=51 time=181ms");
        uow.Out.WriteLine("--- kernel.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 179/181/183 ms");
        uow.Out.WriteLine("net: warning: 198.145.20.140 is not allocated by IANA");
        uow.Out.WriteLine("net: reverse DNS: ftp.kernel.org");
        uow.Out.WriteLine("net: ICMP response banner: \"The Linux Kernel Archives\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("archive.org")]
internal sealed class ArchiveOrg : IEasterEgg
{
    public string Hostname  => "archive.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "207.241.224.2";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING archive.org (207.241.224.2): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 207.241.224.2: icmp_seq=0 ttl=55 time=211ms");
        uow.Out.WriteLine("64 bytes from 207.241.224.2: icmp_seq=1 ttl=55 time=208ms");
        uow.Out.WriteLine("64 bytes from 207.241.224.2: icmp_seq=2 ttl=55 time=201ms");
        uow.Out.WriteLine("--- archive.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 201/206/211 ms");
        uow.Out.WriteLine("net: warning: 207.241.224.2 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP response payload (56 bytes): \"Wayback Machine — saving the web since 1996\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("facebook.com")]
internal sealed class FacebookCom : IEasterEgg
{
    public string Hostname  => "facebook.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "157.240.2.35";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING facebook.com (157.240.2.35): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 157.240.2.35: icmp_seq=0 ttl=56 time=192ms");
        uow.Out.WriteLine("64 bytes from 157.240.2.35: icmp_seq=1 ttl=56 time=187ms");
        uow.Out.WriteLine("64 bytes from 157.240.2.35: icmp_seq=2 ttl=56 time=195ms");
        uow.Out.WriteLine("--- facebook.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 187/191/195 ms");
        uow.Out.WriteLine("net: warning: 157.240.2.35 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP response banner: \"thefacebook.com — a social utility\"");
        uow.Out.WriteLine("net: warning: port 443 responded (SSL not yet standardized)");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("amazon.com")]
internal sealed class AmazonCom : IEasterEgg
{
    public string Hostname  => "amazon.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "205.251.242.103";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING amazon.com (205.251.242.103): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("64 bytes from 205.251.242.103: icmp_seq=1 ttl=48 time=234ms");
        uow.Out.WriteLine("64 bytes from 205.251.242.103: icmp_seq=2 ttl=48 time=229ms");
        uow.Out.WriteLine("--- amazon.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 2 received, 33% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 229/231/234 ms");
        uow.Out.WriteLine("net: warning: 205.251.242.103 is not allocated by IANA");
        uow.Out.WriteLine("net: HTTP 200 on port 80: \"Amazon.com — Earth's Biggest Bookstore\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("youtube.com")]
internal sealed class YoutubeCom : IEasterEgg
{
    public string Hostname  => "youtube.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "142.250.74.110";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING youtube.com (142.250.74.110): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 142.250.74.110: icmp_seq=0 ttl=52 time=217ms");
        uow.Out.WriteLine("64 bytes from 142.250.74.110: icmp_seq=1 ttl=52 time=221ms");
        uow.Out.WriteLine("64 bytes from 142.250.74.110: icmp_seq=2 ttl=52 time=214ms");
        uow.Out.WriteLine("--- youtube.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 214/217/221 ms");
        uow.Out.WriteLine("net: warning: 142.250.74.110 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP response banner: \"Broadcast Yourself\"");
        uow.Out.WriteLine("net: warning: response contains 18,802,501 bytes of streaming video data (truncated to 56)");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("twitter.com")]
internal sealed class TwitterCom : IEasterEgg
{
    public string Hostname  => "twitter.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "104.244.42.193";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING twitter.com (104.244.42.193): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 104.244.42.193: icmp_seq=0 ttl=50 time=199ms");
        uow.Out.WriteLine("64 bytes from 104.244.42.193: icmp_seq=1 ttl=50 time=203ms");
        uow.Out.WriteLine("64 bytes from 104.244.42.193: icmp_seq=2 ttl=50 time=197ms");
        uow.Out.WriteLine("--- twitter.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 197/199/203 ms");
        uow.Out.WriteLine("net: warning: 104.244.42.193 is not allocated by IANA");
        uow.Out.WriteLine("net: warning: ICMP response payload truncated at exactly 140 bytes");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("reddit.com")]
internal sealed class RedditCom : IEasterEgg
{
    public string Hostname  => "reddit.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "151.101.1.140";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING reddit.com (151.101.1.140): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("64 bytes from 151.101.1.140: icmp_seq=1 ttl=53 time=228ms");
        uow.Out.WriteLine("64 bytes from 151.101.1.140: icmp_seq=2 ttl=53 time=222ms");
        uow.Out.WriteLine("--- reddit.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 2 received, 33% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 222/225/228 ms");
        uow.Out.WriteLine("net: warning: 151.101.1.140 is not allocated by IANA");
        uow.Out.WriteLine("net: HTTP banner: \"reddit — the front page of the internet\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("stackoverflow.com")]
internal sealed class StackoverflowCom : IEasterEgg
{
    public string Hostname  => "stackoverflow.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "151.101.65.69";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING stackoverflow.com (151.101.65.69): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 151.101.65.69: icmp_seq=0 ttl=49 time=214ms");
        uow.Out.WriteLine("64 bytes from 151.101.65.69: icmp_seq=1 ttl=49 time=218ms");
        uow.Out.WriteLine("64 bytes from 151.101.65.69: icmp_seq=2 ttl=49 time=211ms");
        uow.Out.WriteLine("--- stackoverflow.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 211/214/218 ms");
        uow.Out.WriteLine("net: warning: 151.101.65.69 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload fragment: \"How do I exit vim?\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("netflix.com")]
internal sealed class NetflixCom : IEasterEgg
{
    public string Hostname  => "netflix.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "52.21.140.173";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING netflix.com (52.21.140.173): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 52.21.140.173: icmp_seq=0 ttl=44 time=241ms");
        uow.Out.WriteLine("64 bytes from 52.21.140.173: icmp_seq=1 ttl=44 time=237ms");
        uow.Out.WriteLine("Request timeout for icmp_seq 2");
        uow.Out.WriteLine("--- netflix.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 2 received, 33% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 237/239/241 ms");
        uow.Out.WriteLine("net: warning: 52.21.140.173 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"Are you still watching?\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("openai.com")]
internal sealed class OpenaiCom : IEasterEgg
{
    public string Hostname  => "openai.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "13.107.238.54";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING openai.com (13.107.238.54): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("Request timeout for icmp_seq 1");
        uow.Out.WriteLine("Request timeout for icmp_seq 2");
        uow.Out.WriteLine("--- openai.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 0 received, 100% packet loss");
        uow.Out.WriteLine("net: error: NXDOMAIN — but route fragment received from AS 8075");
        uow.Out.WriteLine("net: route fragment timestamp: 11 Dec 2015 10:34:22 UTC (inconsistent with local clock)");
        uow.Out.WriteLine("net: fragment payload: \"ensuring AGI benefits all of humanity\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("torproject.org")]
internal sealed class TorprojectOrg : IEasterEgg
{
    public string Hostname  => "torproject.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "116.202.120.166";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING torproject.org (116.202.120.166): 56 data bytes");
        uow.Out.WriteLine("64 bytes from *.*.*.* (anonymised): icmp_seq=0 ttl=? time=?ms");
        uow.Out.WriteLine("64 bytes from *.*.*.* (anonymised): icmp_seq=1 ttl=? time=?ms");
        uow.Out.WriteLine("64 bytes from *.*.*.* (anonymised): icmp_seq=2 ttl=? time=?ms");
        uow.Out.WriteLine("--- torproject.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = [REDACTED]");
        uow.Out.WriteLine("net: warning: route entirely anonymised — 0 of 17 hops visible");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("linkedin.com")]
internal sealed class LinkedinCom : IEasterEgg
{
    public string Hostname  => "linkedin.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "108.174.10.10";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING linkedin.com (108.174.10.10): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 108.174.10.10: icmp_seq=0 ttl=51 time=213ms");
        uow.Out.WriteLine("64 bytes from 108.174.10.10: icmp_seq=1 ttl=51 time=209ms");
        uow.Out.WriteLine("64 bytes from 108.174.10.10: icmp_seq=2 ttl=51 time=211ms");
        uow.Out.WriteLine("--- linkedin.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 209/211/213 ms");
        uow.Out.WriteLine("net: warning: 108.174.10.10 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"You have 1 new connection request\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("slashdot.org")]
internal sealed class SlashdotOrg : IEasterEgg
{
    public string Hostname  => "slashdot.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "216.34.181.45";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING slashdot.org (216.34.181.45): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 216.34.181.45: icmp_seq=0 ttl=46 time=228ms");
        uow.Out.WriteLine("64 bytes from 216.34.181.45: icmp_seq=1 ttl=46 time=224ms");
        uow.Out.WriteLine("64 bytes from 216.34.181.45: icmp_seq=2 ttl=46 time=231ms");
        uow.Out.WriteLine("--- slashdot.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 224/227/231 ms");
        uow.Out.WriteLine("net: warning: 216.34.181.45 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"News for Nerds. Stuff that Matters.\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("sourceforge.net")]
internal sealed class SourceforgeNet : IEasterEgg
{
    public string Hostname  => "sourceforge.net";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "216.105.38.12";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING sourceforge.net (216.105.38.12): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 216.105.38.12: icmp_seq=0 ttl=48 time=233ms");
        uow.Out.WriteLine("64 bytes from 216.105.38.12: icmp_seq=1 ttl=48 time=229ms");
        uow.Out.WriteLine("64 bytes from 216.105.38.12: icmp_seq=2 ttl=48 time=236ms");
        uow.Out.WriteLine("--- sourceforge.net ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 229/232/236 ms");
        uow.Out.WriteLine("net: warning: 216.105.38.12 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"Open Source software development site\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("netscape.com")]
internal sealed class NetscapeCom : IEasterEgg
{
    public string Hostname  => "netscape.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "205.188.153.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING netscape.com (205.188.153.1): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("Request timeout for icmp_seq 1");
        uow.Out.WriteLine("64 bytes from 205.188.153.1: icmp_seq=2 ttl=52 time=247ms");
        uow.Out.WriteLine("--- netscape.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 1 received, 66% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 247/247/247 ms");
        uow.Out.WriteLine("net: warning: 205.188.153.1 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"Netscape Navigator — the browser\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("discord.com")]
internal sealed class DiscordCom : IEasterEgg
{
    public string Hostname  => "discord.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "162.159.128.233";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING discord.com (162.159.128.233): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 162.159.128.233: icmp_seq=0 ttl=55 time=184ms");
        uow.Out.WriteLine("64 bytes from 162.159.128.233: icmp_seq=1 ttl=55 time=188ms");
        uow.Out.WriteLine("64 bytes from 162.159.128.233: icmp_seq=2 ttl=55 time=183ms");
        uow.Out.WriteLine("--- discord.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 183/185/188 ms");
        uow.Out.WriteLine("net: warning: 162.159.128.233 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"imagine having a phone number\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("slack.com")]
internal sealed class SlackCom : IEasterEgg
{
    public string Hostname  => "slack.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "54.192.151.79";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING slack.com (54.192.151.79): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 54.192.151.79: icmp_seq=0 ttl=47 time=236ms");
        uow.Out.WriteLine("64 bytes from 54.192.151.79: icmp_seq=1 ttl=47 time=239ms");
        uow.Out.WriteLine("64 bytes from 54.192.151.79: icmp_seq=2 ttl=47 time=233ms");
        uow.Out.WriteLine("--- slack.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 233/236/239 ms");
        uow.Out.WriteLine("net: warning: 54.192.151.79 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"linus: also ich bin vielleicht kein netter mensch\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("zoom.us")]
internal sealed class ZoomUs : IEasterEgg
{
    public string Hostname  => "zoom.us";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "170.114.0.4";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING zoom.us (170.114.0.4): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 170.114.0.4: icmp_seq=0 ttl=49 time=229ms");
        uow.Out.WriteLine("64 bytes from 170.114.0.4: icmp_seq=1 ttl=49 time=232ms");
        uow.Out.WriteLine("64 bytes from 170.114.0.4: icmp_seq=2 ttl=49 time=227ms");
        uow.Out.WriteLine("--- zoom.us ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 227/229/232 ms");
        uow.Out.WriteLine("net: warning: 170.114.0.4 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"You are now the meeting host\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("instagram.com")]
internal sealed class InstagramCom : IEasterEgg
{
    public string Hostname  => "instagram.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "157.240.3.174";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING instagram.com (157.240.3.174): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 157.240.3.174: icmp_seq=0 ttl=54 time=193ms");
        uow.Out.WriteLine("64 bytes from 157.240.3.174: icmp_seq=1 ttl=54 time=197ms");
        uow.Out.WriteLine("64 bytes from 157.240.3.174: icmp_seq=2 ttl=54 time=191ms");
        uow.Out.WriteLine("--- instagram.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 191/193/197 ms");
        uow.Out.WriteLine("net: warning: 157.240.3.174 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: [image/jpeg 1.2MB — cannot display in terminal]");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("snapchat.com")]
internal sealed class SnapchatCom : IEasterEgg
{
    public string Hostname  => "snapchat.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "35.186.224.47";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING snapchat.com (35.186.224.47): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 35.186.224.47: icmp_seq=0 ttl=50 time=242ms");
        uow.Out.WriteLine("64 bytes from 35.186.224.47: icmp_seq=1 ttl=50 time=238ms");
        uow.Out.WriteLine("64 bytes from 35.186.224.47: icmp_seq=2 ttl=50 time=245ms");
        uow.Out.WriteLine("--- snapchat.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 238/241/245 ms");
        uow.Out.WriteLine("net: warning: 35.186.224.47 is not allocated by IANA");
        uow.Out.WriteLine("net: warning: ICMP response self-destructs after 10 seconds");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("tiktok.com")]
internal sealed class TiktokCom : IEasterEgg
{
    public string Hostname  => "tiktok.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "128.14.149.250";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING tiktok.com (128.14.149.250): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 128.14.149.250: icmp_seq=0 ttl=51 time=354ms");
        uow.Out.WriteLine("64 bytes from 128.14.149.250: icmp_seq=1 ttl=51 time=348ms");
        uow.Out.WriteLine("64 bytes from 128.14.149.250: icmp_seq=2 ttl=51 time=361ms");
        uow.Out.WriteLine("--- tiktok.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 348/354/361 ms");
        uow.Out.WriteLine("net: warning: 128.14.149.250 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: [video/mp4 loop detected — 15 seconds]");
        uow.Out.WriteLine("net: warning: response routed through Beijing (AS 4134) before reaching destination");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("whatsapp.com")]
internal sealed class WhatsappCom : IEasterEgg
{
    public string Hostname  => "whatsapp.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "157.240.8.53";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING whatsapp.com (157.240.8.53): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("64 bytes from 157.240.8.53: icmp_seq=1 ttl=54 time=201ms");
        uow.Out.WriteLine("64 bytes from 157.240.8.53: icmp_seq=2 ttl=54 time=198ms");
        uow.Out.WriteLine("--- whatsapp.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 2 received, 33% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 198/199/201 ms");
        uow.Out.WriteLine("net: warning: 157.240.8.53 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"double tick\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("seti.org")]
internal sealed class SetiOrg : IEasterEgg
{
    public string Hostname  => "seti.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "207.218.253.51";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING seti.org (207.218.253.51): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("Request timeout for icmp_seq 1");
        uow.Out.WriteLine("Request timeout for icmp_seq 2");
        uow.Out.WriteLine("--- seti.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 0 received, 100% packet loss");
        uow.Out.WriteLine("net: error: no route to host");
        uow.Out.WriteLine("net: note: 1 packet returned from unknown AS — origin undefined");
        uow.Out.WriteLine("net: ICMP payload: \"...WOW!\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("creativecommons.org")]
internal sealed class CreativecommonsOrg : IEasterEgg
{
    public string Hostname  => "creativecommons.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "54.84.12.12";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING creativecommons.org (54.84.12.12): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 54.84.12.12: icmp_seq=0 ttl=50 time=219ms");
        uow.Out.WriteLine("64 bytes from 54.84.12.12: icmp_seq=1 ttl=50 time=222ms");
        uow.Out.WriteLine("64 bytes from 54.84.12.12: icmp_seq=2 ttl=50 time=218ms");
        uow.Out.WriteLine("--- creativecommons.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 218/219/222 ms");
        uow.Out.WriteLine("net: warning: 54.84.12.12 is not allocated by IANA");
        uow.Out.WriteLine("net: ICMP payload: \"This ping response is licensed CC BY-SA 4.0\"");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("y2k.com")]
internal sealed class Y2kCom : IEasterEgg
{
    public string Hostname  => "y2k.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "192.168.1.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING y2k.com (192.168.1.1): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 192.168.1.1: icmp_seq=0 ttl=64 time=0.01ms");
        uow.Out.WriteLine("64 bytes from 192.168.1.1: icmp_seq=1 ttl=64 time=0.01ms");
        uow.Out.WriteLine("64 bytes from 192.168.1.1: icmp_seq=2 ttl=64 time=0.01ms");
        uow.Out.WriteLine("--- y2k.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 0.00/0.00/0.01 ms");
        uow.Out.WriteLine("net: warning: response from 192.168.1.1 (RFC 1918 private space — unroutable)");
        uow.Out.WriteLine("net: timestamp in ICMP response: 00:00:00.001 Jan 1 2000");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("void.null")]
internal sealed class VoidNull : IEasterEgg
{
    public string Hostname  => "void.null";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING void.null (): 56 data bytes");
        uow.Out.WriteLine("64 bytes from : icmp_seq=0 ttl=0 time=-1ms");
        uow.Out.WriteLine("64 bytes from : icmp_seq=1 ttl=0 time=-1ms");
        uow.Out.WriteLine("64 bytes from : icmp_seq=2 ttl=0 time=-1ms");
        uow.Out.WriteLine("--- void.null ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = -1/-1/-1 ms");
        uow.Out.WriteLine("net: error: negative latency detected — check system clock");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

// The funny one — pinging your own future self
[RemoteHost("linus.torvalds.name")]
internal sealed class LinusTorvaldsName : IEasterEgg
{
    public string Hostname  => "linus.torvalds.name";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "127.0.0.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING linus.torvalds.name (127.0.0.1): 56 data bytes");
        uow.Out.WriteLine("64 bytes from linus.torvalds.name (127.0.0.1): icmp_seq=0 ttl=64 time=0.00ms");
        uow.Out.WriteLine("64 bytes from linus.torvalds.name (127.0.0.1): icmp_seq=1 ttl=64 time=0.00ms");
        uow.Out.WriteLine("64 bytes from linus.torvalds.name (127.0.0.1): icmp_seq=2 ttl=64 time=0.00ms");
        uow.Out.WriteLine("--- linus.torvalds.name ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 0.00/0.00/0.00 ms");
        uow.Out.WriteLine("net: reverse DNS resolves to: you");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("unknown.global")]
internal sealed class UnknownGlobal : IEasterEgg
{
    public string Hostname  => "unknown.global";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING unknown.global (): 56 data bytes");
        uow.Out.WriteLine("net: error: NXDOMAIN");
        uow.Out.WriteLine("net: error: reverse lookup failed");
        uow.Out.WriteLine("net: error: no route to host");
        uow.Out.WriteLine("net: warning: partial route trace received from unallocated AS");
        uow.Out.WriteLine("net: warning: this host should not exist");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}


// === Phase 3: 30 new sophisticated domain-specific anomalous hosts ===

[RemoteHost("stripe.com")]
internal sealed class StripeCom : IEasterEgg
{
    public string Hostname  => "stripe.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "52.89.214.238";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING stripe.com (52.89.214.238): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 52.89.214.238: icmp_seq=0 ttl=53 time=62ms");
        uow.Out.WriteLine("64 bytes from 52.89.214.238: icmp_seq=1 ttl=53 time=61ms");
        uow.Out.WriteLine("64 bytes from 52.89.214.238: icmp_seq=2 ttl=53 time=63ms");
        uow.Out.WriteLine("--- stripe.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 61/62/63 ms");
        uow.Out.WriteLine("net: ICMP payload charged $0.01 to account");
        uow.Out.WriteLine("net: warning: this ping cannot be refunded");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("paypal.com")]
internal sealed class PaypalCom : IEasterEgg
{
    public string Hostname  => "paypal.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "173.0.85.101";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING paypal.com (173.0.85.101): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 173.0.85.101: icmp_seq=0 ttl=53 time=47ms");
        uow.Out.WriteLine("64 bytes from 173.0.85.101: icmp_seq=1 ttl=53 time=48ms");
        uow.Out.WriteLine("64 bytes from 173.0.85.101: icmp_seq=2 ttl=53 time=49ms");
        uow.Out.WriteLine("--- paypal.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 47/48/49 ms");
        uow.Out.WriteLine("net: transaction ID for ping not found");
        uow.Out.WriteLine("net: account frozen — verify identity");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("telegram.org")]
internal sealed class TelegramOrg : IEasterEgg
{
    public string Hostname  => "telegram.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "149.154.167.99";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING telegram.org (149.154.167.99): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("64 bytes from 149.154.167.99: icmp_seq=1 ttl=53 time=71ms");
        uow.Out.WriteLine("Request timeout for icmp_seq 2");
        uow.Out.WriteLine("--- telegram.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 1 received, 66% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 71/71/71 ms");
        uow.Out.WriteLine("net: responses encrypted (unable to verify)");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("signal.org")]
internal sealed class SignalOrg : IEasterEgg
{
    public string Hostname  => "signal.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "151.101.1.140";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING signal.org (151.101.1.140): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 151.101.1.140: icmp_seq=0 ttl=52 time=73ms");
        uow.Out.WriteLine("64 bytes from 151.101.1.140: icmp_seq=1 ttl=52 time=74ms");
        uow.Out.WriteLine("64 bytes from 151.101.1.140: icmp_seq=2 ttl=52 time=72ms");
        uow.Out.WriteLine("--- signal.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 72/73/74 ms");
        uow.Out.WriteLine("net: all responses encrypted with public key");
        uow.Out.WriteLine("net: decrypt key found in RAM from 48 hours ago");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("matrix.org")]
internal sealed class MatrixOrg : IEasterEgg
{
    public string Hostname  => "matrix.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "45.76.99.226";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING matrix.org (45.76.99.226): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 45.76.99.226: icmp_seq=0 ttl=56 time=82ms");
        uow.Out.WriteLine("64 bytes from 45.76.99.226: icmp_seq=1 ttl=56 time=81ms");
        uow.Out.WriteLine("64 bytes from 45.76.99.226: icmp_seq=2 ttl=56 time=83ms");
        uow.Out.WriteLine("--- matrix.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 81/82/83 ms");
        uow.Out.WriteLine("net: federated across 47 homeservers");
        uow.Out.WriteLine("net: consensus delay included in latency");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("bbc.com")]
internal sealed class BbcCom : IEasterEgg
{
    public string Hostname  => "bbc.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "212.58.244.70";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING bbc.com (212.58.244.70): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 212.58.244.70: icmp_seq=0 ttl=52 time=9ms");
        uow.Out.WriteLine("64 bytes from 212.58.244.70: icmp_seq=1 ttl=52 time=9ms");
        uow.Out.WriteLine("64 bytes from 212.58.244.70: icmp_seq=2 ttl=52 time=9ms");
        uow.Out.WriteLine("--- bbc.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 9/9/9 ms");
        uow.Out.WriteLine("net: breaking news from 3 days ahead");
        uow.Out.WriteLine("net: warning: spoilers for your region");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("reuters.com")]
internal sealed class ReutersCom : IEasterEgg
{
    public string Hostname  => "reuters.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "213.52.136.140";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING reuters.com (213.52.136.140): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 213.52.136.140: icmp_seq=0 ttl=52 time=18ms");
        uow.Out.WriteLine("64 bytes from 213.52.136.140: icmp_seq=1 ttl=52 time=19ms");
        uow.Out.WriteLine("64 bytes from 213.52.136.140: icmp_seq=2 ttl=52 time=17ms");
        uow.Out.WriteLine("--- reuters.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 17/18/19 ms");
        uow.Out.WriteLine("net: ICMP response retracted");
        uow.Out.WriteLine("net: timestamp 20 min before local clock");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("dropbox.com")]
internal sealed class DropboxCom : IEasterEgg
{
    public string Hostname  => "dropbox.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "162.125.74.36";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING dropbox.com (162.125.74.36): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 162.125.74.36: icmp_seq=0 ttl=53 time=34ms");
        uow.Out.WriteLine("64 bytes from 162.125.74.36: icmp_seq=1 ttl=53 time=35ms");
        uow.Out.WriteLine("64 bytes from 162.125.74.36: icmp_seq=2 ttl=53 time=33ms");
        uow.Out.WriteLine("--- dropbox.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 33/34/35 ms");
        uow.Out.WriteLine("net: synced to 7 devices you don't own");
        uow.Out.WriteLine("net: shared with 12 contacts automatically");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("aws.amazon.com")]
internal sealed class AwsAmazonCom : IEasterEgg
{
    public string Hostname  => "aws.amazon.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "176.32.98.166";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING aws.amazon.com (176.32.98.166): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 176.32.98.166: icmp_seq=0 ttl=53 time=89ms");
        uow.Out.WriteLine("64 bytes from 176.32.98.166: icmp_seq=1 ttl=53 time=102ms");
        uow.Out.WriteLine("64 bytes from 176.32.98.166: icmp_seq=2 ttl=53 time=94ms");
        uow.Out.WriteLine("--- aws.amazon.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 89/95/102 ms");
        uow.Out.WriteLine("net: billing: $0.00001 per millisecond");
        uow.Out.WriteLine("net: session bill: $0.00285");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("gitlab.com")]
internal sealed class GitlabCom : IEasterEgg
{
    public string Hostname  => "gitlab.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "172.65.251.78";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING gitlab.com (172.65.251.78): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 172.65.251.78: icmp_seq=0 ttl=47 time=19ms");
        uow.Out.WriteLine("64 bytes from 172.65.251.78: icmp_seq=1 ttl=47 time=18ms");
        uow.Out.WriteLine("64 bytes from 172.65.251.78: icmp_seq=2 ttl=47 time=20ms");
        uow.Out.WriteLine("--- gitlab.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 18/19/20 ms");
        uow.Out.WriteLine("net: fork detected — 4 conflicting versions");
        uow.Out.WriteLine("net: merge conflict in icmp_seq=1");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("bing.com")]
internal sealed class BingCom : IEasterEgg
{
    public string Hostname  => "bing.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "204.79.197.200";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING bing.com (204.79.197.200): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 204.79.197.200: icmp_seq=0 ttl=54 time=23ms");
        uow.Out.WriteLine("64 bytes from 204.79.197.200: icmp_seq=1 ttl=54 time=24ms");
        uow.Out.WriteLine("64 bytes from 204.79.197.200: icmp_seq=2 ttl=54 time=23ms");
        uow.Out.WriteLine("--- bing.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 23/23/24 ms");
        uow.Out.WriteLine("net: Did you mean: google.com?");
        uow.Out.WriteLine("net: suggesting different target");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("duckduckgo.com")]
internal sealed class DuckduckgoCom : IEasterEgg
{
    public string Hostname  => "duckduckgo.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "3.213.240.89";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING duckduckgo.com (3.213.240.89): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 3.213.240.89: icmp_seq=0 ttl=54 time=41ms");
        uow.Out.WriteLine("64 bytes from 3.213.240.89: icmp_seq=1 ttl=54 time=42ms");
        uow.Out.WriteLine("64 bytes from 3.213.240.89: icmp_seq=2 ttl=54 time=40ms");
        uow.Out.WriteLine("--- duckduckgo.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 40/41/42 ms");
        uow.Out.WriteLine("net: We didn't log your ping");
        uow.Out.WriteLine("net: payload not tracked");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("microsoft.com")]
internal sealed class MicrosoftCom : IEasterEgg
{
    public string Hostname  => "microsoft.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "13.107.42.14";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING microsoft.com (13.107.42.14): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 13.107.42.14: icmp_seq=0 ttl=56 time=16ms");
        uow.Out.WriteLine("64 bytes from 13.107.42.14: icmp_seq=1 ttl=56 time=16ms");
        uow.Out.WriteLine("64 bytes from 13.107.42.14: icmp_seq=2 ttl=56 time=16ms");
        uow.Out.WriteLine("--- microsoft.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 16/16/16 ms");
        uow.Out.WriteLine("net: 412 critical updates in payload");
        uow.Out.WriteLine("net: reboot required before next ping");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("apple.com")]
internal sealed class AppleCom : IEasterEgg
{
    public string Hostname  => "apple.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "17.142.160.59";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING apple.com (17.142.160.59): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 17.142.160.59: icmp_seq=0 ttl=51 time=74ms");
        uow.Out.WriteLine("64 bytes from 17.142.160.59: icmp_seq=1 ttl=51 time=75ms");
        uow.Out.WriteLine("64 bytes from 17.142.160.59: icmp_seq=2 ttl=51 time=73ms");
        uow.Out.WriteLine("--- apple.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 73/74/75 ms");
        uow.Out.WriteLine("net: only works on Apple hardware");
        uow.Out.WriteLine("net: requires iTunes running");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("cloudflare.com")]
internal sealed class CloudflareCom : IEasterEgg
{
    public string Hostname  => "cloudflare.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "104.16.132.229";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING cloudflare.com (104.16.132.229): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 104.16.132.229: icmp_seq=0 ttl=53 time=13ms");
        uow.Out.WriteLine("64 bytes from 104.16.132.229: icmp_seq=1 ttl=53 time=13ms");
        uow.Out.WriteLine("64 bytes from 104.16.132.229: icmp_seq=2 ttl=53 time=13ms");
        uow.Out.WriteLine("--- cloudflare.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 13/13/13 ms");
        uow.Out.WriteLine("net: cached from 73 datacenters");
        uow.Out.WriteLine("net: served from anycast — origin unknown");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("heroku.com")]
internal sealed class HerokoCom : IEasterEgg
{
    public string Hostname  => "heroku.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "54.175.233.142";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING heroku.com (54.175.233.142): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 54.175.233.142: icmp_seq=0 ttl=55 time=41ms");
        uow.Out.WriteLine("64 bytes from 54.175.233.142: icmp_seq=1 ttl=55 time=40ms");
        uow.Out.WriteLine("64 bytes from 54.175.233.142: icmp_seq=2 ttl=55 time=42ms");
        uow.Out.WriteLine("--- heroku.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 40/41/42 ms");
        uow.Out.WriteLine("net: dyno restarted — response rebuilt");
        uow.Out.WriteLine("net: cold start latency on 3rd ping");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("ethereum.org")]
internal sealed class EthereumOrg : IEasterEgg
{
    public string Hostname  => "ethereum.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "104.21.10.74";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING ethereum.org (104.21.10.74): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 104.21.10.74: icmp_seq=0 ttl=54 time=127ms");
        uow.Out.WriteLine("64 bytes from 104.21.10.74: icmp_seq=1 ttl=54 time=128ms");
        uow.Out.WriteLine("64 bytes from 104.21.10.74: icmp_seq=2 ttl=54 time=126ms");
        uow.Out.WriteLine("--- ethereum.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 126/127/128 ms");
        uow.Out.WriteLine("net: consensus from 7 validators");
        uow.Out.WriteLine("net: gas fee: 0.0012 ETH");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("spotify.com")]
internal sealed class SpotifyCom : IEasterEgg
{
    public string Hostname  => "spotify.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "35.195.14.250";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING spotify.com (35.195.14.250): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 35.195.14.250: icmp_seq=0 ttl=49 time=36ms");
        uow.Out.WriteLine("64 bytes from 35.195.14.250: icmp_seq=1 ttl=49 time=37ms");
        uow.Out.WriteLine("64 bytes from 35.195.14.250: icmp_seq=2 ttl=49 time=35ms");
        uow.Out.WriteLine("--- spotify.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 35/36/37 ms");
        uow.Out.WriteLine("net: ICMP skipping to next response");
        uow.Out.WriteLine("net: marked 'do not ping again'");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("medium.com")]
internal sealed class MediumCom : IEasterEgg
{
    public string Hostname  => "medium.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "35.186.202.80";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING medium.com (35.186.202.80): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 35.186.202.80: icmp_seq=0 ttl=49 time=129ms");
        uow.Out.WriteLine("64 bytes from 35.186.202.80: icmp_seq=1 ttl=49 time=131ms");
        uow.Out.WriteLine("64 bytes from 35.186.202.80: icmp_seq=2 ttl=49 time=128ms");
        uow.Out.WriteLine("--- medium.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 128/129/131 ms");
        uow.Out.WriteLine("net: Response: 10 min read");
        uow.Out.WriteLine("net: paywall active — subscribe");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("notion.so")]
internal sealed class NotionSo : IEasterEgg
{
    public string Hostname  => "notion.so";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "104.18.8.97";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING notion.so (104.18.8.97): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 104.18.8.97: icmp_seq=0 ttl=54 time=147ms");
        uow.Out.WriteLine("64 bytes from 104.18.8.97: icmp_seq=1 ttl=54 time=1847ms");
        uow.Out.WriteLine("64 bytes from 104.18.8.97: icmp_seq=2 ttl=54 time=148ms");
        uow.Out.WriteLine("--- notion.so ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 147/1047/1847 ms");
        uow.Out.WriteLine("net: icmp_seq=1 timeout then resolved");
        uow.Out.WriteLine("net: loading spinner still visible");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("protonmail.com")]
internal sealed class ProtonmailCom : IEasterEgg
{
    public string Hostname  => "protonmail.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "185.70.40.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING protonmail.com (185.70.40.1): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 185.70.40.1: icmp_seq=0 ttl=52 time=67ms");
        uow.Out.WriteLine("64 bytes from 185.70.40.1: icmp_seq=1 ttl=52 time=68ms");
        uow.Out.WriteLine("64 bytes from 185.70.40.1: icmp_seq=2 ttl=52 time=66ms");
        uow.Out.WriteLine("--- protonmail.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 66/67/68 ms");
        uow.Out.WriteLine("net: all responses encrypted end-to-end");
        uow.Out.WriteLine("net: server cannot read payload");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("fastmail.com")]
internal sealed class FastmailCom : IEasterEgg
{
    public string Hostname  => "fastmail.com";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "103.105.40.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING fastmail.com (103.105.40.1): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 103.105.40.1: icmp_seq=0 ttl=58 time=8ms");
        uow.Out.WriteLine("64 bytes from 103.105.40.1: icmp_seq=1 ttl=58 time=7ms");
        uow.Out.WriteLine("64 bytes from 103.105.40.1: icmp_seq=2 ttl=58 time=9ms");
        uow.Out.WriteLine("--- fastmail.com ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 7/8/9 ms");
        uow.Out.WriteLine("net: delivered to 47 aliases");
        uow.Out.WriteLine("net: responses merged and deduplicated");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("bitbucket.org")]
internal sealed class BitbucketOrg : IEasterEgg
{
    public string Hostname  => "bitbucket.org";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "104.192.141.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING bitbucket.org (104.192.141.1): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 104.192.141.1: icmp_seq=0 ttl=52 time=31ms");
        uow.Out.WriteLine("64 bytes from 104.192.141.1: icmp_seq=1 ttl=52 time=30ms");
        uow.Out.WriteLine("64 bytes from 104.192.141.1: icmp_seq=2 ttl=52 time=32ms");
        uow.Out.WriteLine("--- bitbucket.org ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 30/31/32 ms");
        uow.Out.WriteLine("net: ICMP forked internally");
        uow.Out.WriteLine("net: responses contain git history");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("mastodon.social")]
internal sealed class MastodonSocial : IEasterEgg
{
    public string Hostname  => "mastodon.social";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "104.21.12.92";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING mastodon.social (104.21.12.92): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 104.21.12.92: icmp_seq=0 ttl=54 time=156ms");
        uow.Out.WriteLine("64 bytes from 104.21.12.92: icmp_seq=1 ttl=54 time=157ms");
        uow.Out.WriteLine("64 bytes from 104.21.12.92: icmp_seq=2 ttl=54 time=155ms");
        uow.Out.WriteLine("--- mastodon.social ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 155/156/157 ms");
        uow.Out.WriteLine("net: federated across 8000+ instances");
        uow.Out.WriteLine("net: payload has remote metadata");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("lobste.rs")]
internal sealed class LobstersRs : IEasterEgg
{
    public string Hostname  => "lobste.rs";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "108.165.75.39";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING lobste.rs (108.165.75.39): 56 data bytes");
        uow.Out.WriteLine("64 bytes from 108.165.75.39: icmp_seq=0 ttl=54 time=71ms");
        uow.Out.WriteLine("64 bytes from 108.165.75.39: icmp_seq=1 ttl=54 time=70ms");
        uow.Out.WriteLine("64 bytes from 108.165.75.39: icmp_seq=2 ttl=54 time=72ms");
        uow.Out.WriteLine("--- lobste.rs ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 3 received, 0% packet loss");
        uow.Out.WriteLine("round-trip min/avg/max = 70/71/72 ms");
        uow.Out.WriteLine("net: response flagged as offtopic");
        uow.Out.WriteLine("net: sent to moderation queue");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}

[RemoteHost("twtxt.net")]
internal sealed class TwtxtNet : IEasterEgg
{
    public string Hostname  => "twtxt.net";
    public IReadOnlyList<string> Aliases => [];
    public string IpAddress => "192.0.2.1";
    public int BasePingMs   => 0;
    public HostAccess Access => HostAccess.Normal;

    public void Execute(IUnitOfWork uow)
    {
        uow.Out.WriteLine("PING twtxt.net (192.0.2.1): 56 data bytes");
        uow.Out.WriteLine("Request timeout for icmp_seq 0");
        uow.Out.WriteLine("Request timeout for icmp_seq 1");
        uow.Out.WriteLine("Request timeout for icmp_seq 2");
        uow.Out.WriteLine("--- twtxt.net ping statistics ---");
        uow.Out.WriteLine("3 packets transmitted, 0 received, 100% packet loss");
        uow.Out.WriteLine("net: no route to host (never existed)");
        uow.Out.WriteLine("net: 2014 decentralized dream");
        uow.Out.WriteLine("net: note: anomaly logged to /usr/adm/net.trace");
    }
}
