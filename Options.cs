namespace AudioBrowser;

public class Options
{
    [Obsolete($"Do not use this, instead use {nameof(FilesDirectory)}")]
    [ConfigurationKeyName("FilesPath")]
    public required string FilesPath { get; init; }

#pragma warning disable CS0618 // Type or member is obsolete
    public virtual DirectoryInfo FilesDirectory => new(FilesPath);
#pragma warning restore CS0618 // Type or member is obsolete
}