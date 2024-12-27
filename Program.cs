using System.Net.Mime;
using AudioBrowser.Components;
using AudioBrowser.Services;
using Microsoft.AspNetCore.DataProtection;
using Microsoft.AspNetCore.Mvc;
using Microsoft.AspNetCore.StaticFiles;
using Microsoft.Extensions.Options;
using Options = AudioBrowser.Options;

// new FileInfo("/tmp/test.txt").Create();
// XAttrsService.SetBool(new FileInfo("/tmp/test.txt"), "test", true);
// Console.WriteLine($"Woot it worked: {XAttrsService.GetBool(new FileInfo("/tmp/test.txt"), "test")}");

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddDataProtection().PersistKeysToFileSystem(new DirectoryInfo("/files"));
builder.Services.Configure<Options>(builder.Configuration);

// Add services to the container.
builder.Services.AddRazorComponents()
    .AddInteractiveServerComponents();

builder.Services.AddHostedService<WatcherService>();

var app = builder.Build();

// Configure the HTTP request pipeline.
if (!app.Environment.IsDevelopment())
{
    app.UseExceptionHandler("/Error", createScopeForErrors: true);
    // The default HSTS value is 30 days. You may want to change this for production scenarios, see https://aka.ms/aspnetcore-hsts.
    app.UseHsts();
}

app.UseHttpsRedirection();


app.UseAntiforgery();

app.MapStaticAssets();
app.MapRazorComponents<App>()
    .AddInteractiveServerRenderMode();

app.MapGet("/file/{**path}", ([FromRoute] string path, [FromServices] IOptions<Options> options, [FromServices] ILogger<Program> logger) =>
{
    var file = new FileInfo(Path.Join(options.Value.FilesDirectory.FullName, path));
    if (!file.FullName.StartsWith(options.Value.FilesDirectory.FullName)) return Results.BadRequest();

    logger.LogInformation("File request {} {} {}", path, file.FullName, file.Exists);
    
    if (!file.Exists) return Results.NotFound();
    var contentTypeMapping = new FileExtensionContentTypeProvider();
    _ = contentTypeMapping.TryGetContentType(file.Name, out var type);

    return Results.File(file.OpenRead(), type, file.Name);
});

app.Run();