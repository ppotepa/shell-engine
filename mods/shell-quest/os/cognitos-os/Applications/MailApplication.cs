namespace CognitosOs.Applications;

using CognitosOs.Core;
using CognitosOs.Kernel;

internal sealed class MailApplication : IKernelApplication
{
    private int _currentIndex;

    public string PromptPrefix(UserSession session) => "mail> ";

    public void OnEnter(IUnitOfWork uow)
    {
        if (uow.Mail.List().Count == 0)
        {
            uow.Out.WriteLine("No mail.");
            return;
        }

        ShowCurrent(uow);
    }

    public void OnExit(IUnitOfWork uow) { }

    public ApplicationResult HandleInput(IUnitOfWork uow, string input)
    {
        var cmd = string.IsNullOrWhiteSpace(input) ? "" : input.Trim();
        switch (cmd)
        {
            case "":
            case "p":
                ShowCurrent(uow);
                return ApplicationResult.Continue;
            case "d":
                DeleteCurrent(uow);
                return ApplicationResult.Continue;
            case "q":
            case "x":
                return ApplicationResult.Exit;
            default:
                uow.Out.WriteLine("Commands: <enter>, p, d, q, x");
                return ApplicationResult.Continue;
        }
    }

    private void ShowCurrent(IUnitOfWork uow)
    {
        var entry = uow.Mail.Read(_currentIndex);
        if (entry is null)
        {
            uow.Out.WriteLine("No more mail.");
            return;
        }

        uow.Mail.MarkRead(_currentIndex);
        uow.Out.WriteLine($"From {entry.From} {entry.Date:ddd MMM dd HH:mm:ss yyyy}");
        uow.Out.WriteLine($"Subject: {entry.Subject}");
        uow.Out.WriteLine();
        uow.Out.WriteLine(entry.Body);
    }

    private void DeleteCurrent(IUnitOfWork uow)
    {
        uow.Out.WriteLine("Message marked for deletion.");
        _currentIndex++;
        ShowCurrent(uow);
    }
}
