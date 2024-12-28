using Microsoft.Extensions.Options;

namespace AudioBrowser.Services;

public class WatcherService(IOptionsMonitor<Options> optionsMonitor, ILogger<WatcherService> logger) : IHostedService
{
    private FileSystemWatcher _watcher = null!;

    public static event EventHandler<EventArgs>? FilesChanged;
    
    public Task StartAsync(CancellationToken cancellationToken)
    {
        logger.LogInformation("Watching files in {Path}", optionsMonitor.CurrentValue.FilesDirectory.FullName);
        _watcher = new FileSystemWatcher(optionsMonitor.CurrentValue.FilesDirectory.FullName)
        {
            IncludeSubdirectories = true,
            Filter = "*.mp3",
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
            if (evt.Name?.EndsWith("tmp") ?? false) return;
            
            FilesChanged?.Invoke(null, EventArgs.Empty);
        };
        
        _watcher.Renamed += (_, evt) =>
        {
            logger.LogInformation("File renamed: {} {}", evt.FullPath, evt.ChangeType);
            if (evt.Name?.EndsWith("tmp") ?? false) return;
            
            FilesChanged?.Invoke(null, EventArgs.Empty);
        };
        
        _watcher.Deleted += (_, evt) =>
        {
            logger.LogInformation("File deleted: {} {}", evt.FullPath, evt.ChangeType);
            if (evt.Name?.EndsWith("tmp") ?? false) return;
            
            FilesChanged?.Invoke(null, EventArgs.Empty);
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