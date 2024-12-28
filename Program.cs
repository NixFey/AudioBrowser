using AudioBrowser.Components;
using AudioBrowser.Services;
using Microsoft.AspNetCore.DataProtection;
using Microsoft.Extensions.FileProviders;
using Microsoft.Extensions.Options;
using Options = AudioBrowser.Options;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddDataProtection().PersistKeysToFileSystem(new DirectoryInfo(builder.Configuration["FilesPath"] ??
                                                                               throw new ApplicationException(
                                                                                   "FilesPath configuration not specified")));
builder.Services.Configure<Options>(builder.Configuration);

// Add services to the container.
builder.Services.AddRazorComponents()
    .AddInteractiveServerComponents();

builder.Services.AddSingleton<WatcherService>();
builder.Services.AddHostedService<WatcherService>();

builder.Services.AddOptions<StaticFileOptions>().Configure<IOptionsMonitor<Options>>((sfOpt, opt) =>
{
    sfOpt.RequestPath = "/file";
    sfOpt.FileProvider = new PhysicalFileProvider(opt.CurrentValue.FilesDirectory.FullName);
});

var app = builder.Build();

// Configure the HTTP request pipeline.
if (!app.Environment.IsDevelopment())
{
    app.UseExceptionHandler("/Error", createScopeForErrors: true);
    // The default HSTS value is 30 days. You may want to change this for production scenarios, see https://aka.ms/aspnetcore-hsts.
    app.UseHsts();
}

app.UseAntiforgery();

app.MapStaticAssets();
app.MapRazorComponents<App>()
    .AddInteractiveServerRenderMode();

// Note: If the files directory is changed, this won't pick it up without a full restart of the app
app.UseStaticFiles();

app.Run();