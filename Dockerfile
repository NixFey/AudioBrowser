FROM mcr.microsoft.com/dotnet/aspnet:9.0 AS base
USER $APP_UID
ARG TARGETARCH
WORKDIR /app
EXPOSE 8080
EXPOSE 8081

FROM --platform=$BUILDPLATFORM mcr.microsoft.com/dotnet/sdk:9.0 AS build
ARG BUILD_CONFIGURATION=Release
WORKDIR /src
COPY ["AudioBrowser.csproj", "./"]
RUN dotnet restore "AudioBrowser.csproj" -a $TARGETARCH
COPY . .
WORKDIR "/src/"
#RUN dotnet build -a $TARGETARCH --no-restore "AudioBrowser.csproj" -c $BUILD_CONFIGURATION -o /app/build

FROM build AS publish
ARG BUILD_CONFIGURATION=Release
RUN dotnet publish -a $TARGETARCH --no-restore "AudioBrowser.csproj" -c $BUILD_CONFIGURATION -o /app/publish

FROM base AS final
WORKDIR /app
COPY --from=publish /app/publish .
ENTRYPOINT ["./AudioBrowser"]
