using Microsoft.Extensions.Options;

namespace AudioBrowser.Services;

public class WatcherService(IOptionsMonitor<Options> optionsMonitor, ILogger<WatcherService> logger) : IHostedService
{
    private FileSystemWatcher _watcher;
    public Task StartAsync(CancellationToken cancellationToken)
    {
        logger.LogInformation("aaWatching files in {Path}", optionsMonitor.CurrentValue.FilesDirectory.FullName);
        _watcher = new FileSystemWatcher(optionsMonitor.CurrentValue.FilesDirectory.FullName)
        {
            IncludeSubdirectories = true,
            NotifyFilter = NotifyFilters.Attributes |
                           NotifyFilters.CreationTime |
                           NotifyFilters.FileName |
                           NotifyFilters.LastAccess |
                           NotifyFilters.LastWrite |
                           NotifyFilters.Size |
                           NotifyFilters.Security
        };

        _watcher.Created += (_, evt) =>
        {
            logger.LogInformation("File created: {} {}", evt.FullPath, evt.ChangeType);
        };
        
        _watcher.Renamed += (_, evt) =>
        {
            logger.LogInformation("File created: {} {}", evt.FullPath, evt.ChangeType);
        };

        _watcher.EnableRaisingEvents = true;

        return Task.CompletedTask;
    }

    public Task StopAsync(CancellationToken cancellationToken)
    {
        _watcher.Dispose();
        return Task.CompletedTask;
    }
}