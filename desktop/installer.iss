; Inno Setup Script for LinkCam Windows Companion
#define MyAppName "LinkCam"
#define MyAppVersion "1.2.3"
#define MyAppPublisher "LinkCam Team"
#define MyAppURL "https://github.com/Aniketkumar-01/LinkCam"
#define MyAppExeName "linkcam-desktop.exe"

[Setup]
AppId={{D37E5526-92F7-4E61-A76F-B590E8F88E28}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
OutputDir=Output
OutputBaseFilename=LinkCam-Windows-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
CloseApplications=yes
RestartApplications=no
CloseApplicationsFilter=*.exe,*.dll

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "startupicon"; Description: "Launch LinkCam automatically on Windows startup"; GroupDescription: "Startup options:"; Flags: unchecked

[Files]
Source: "src-tauri\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion restartreplace
Source: "src-tauri\target\release\LinkCamDShow64.dll"; DestDir: "{app}"; Flags: ignoreversion restartreplace uninsrestartdelete
Source: "dist\*"; DestDir: "{app}\dist"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon
Name: "{userstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: startupicon

[Run]
Filename: "regsvr32.exe"; Parameters: "/s ""{app}\LinkCamDShow64.dll"""; StatusMsg: "Registering Virtual Camera Filter..."; Flags: runhidden
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "taskkill.exe"; Parameters: "/f /im {#MyAppExeName}"; Flags: runhidden; RunOnceId: "KillApp"
Filename: "regsvr32.exe"; Parameters: "/u /s ""{app}\LinkCamDShow64.dll"""; StatusMsg: "Unregistering Virtual Camera Filter..."; Flags: runhidden; RunOnceId: "UnregFilter"

[UninstallDelete]
Type: filesandordirs; Name: "{localappdata}\LinkCam"
Type: filesandordirs; Name: "{localappdata}\com.linkcam.desktop"
Type: files; Name: "{app}\*.log"
Type: dirifempty; Name: "{app}\dist"
Type: dirifempty; Name: "{app}"

[Code]
// Proactively terminate running companion and release camera server handles before setup
function InitializeSetup(): Boolean;
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/f /im {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec('net.exe', 'stop FrameServer', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := True;
end;

// Proactively terminate running companion, unregister DLL out-of-process, and release camera handles
function InitializeUninstall(): Boolean;
var
  ResultCode: Integer;
begin
  Exec('taskkill.exe', '/f /im {#MyAppExeName}', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec('regsvr32.exe', '/u /s "' + ExpandConstant('{app}\LinkCamDShow64.dll') + '"', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Exec('net.exe', 'stop FrameServer', '', SW_HIDE, ewWaitUntilTerminated, ResultCode);
  Result := True;
end;
