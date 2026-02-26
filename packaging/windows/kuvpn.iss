#define MyAppName "KUVPN"
#define MyAppVersion "2.0.2"
#define MyAppPublisher "Koç University"
#define MyAppURL "https://github.com/nyverino/kuvpn"
#define MyAppExeName "kuvpn-gui.exe"

[Setup]
; NOTE: The value of AppId uniquely identifies this application.
; Do not use the same AppId value in installers for other applications.
; (To generate a new GUID, click Tools | Generate GUID inside the IDE.)
AppId={{D3B2A1E9-4B5C-4F7D-9E8A-9A2B6C5D4E3F}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
;AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DisableProgramGroupPage=yes
; Uncomment the following line to run in non administrative install mode (install for current user only.)
;PrivilegesRequired=lowest
PrivilegesRequired=admin
OutputDir=.
OutputBaseFilename=KUVPN-Setup
SetupIconFile=..\..\crates\kuvpn-gui\assets\icon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
Compression=lzma
SolidCompression=yes
WizardStyle=modern

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "..\..\target\x86_64-pc-windows-gnu\release\kuvpn-gui.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\..\target\x86_64-pc-windows-gnu\release\kuvpn-win-helper.exe"; DestDir: "{app}"; Flags: ignoreversion
; Bundle OpenConnect binaries if present in the 'openconnect' subdirectory
Source: "openconnect\*"; DestDir: "{app}\openconnect"; Flags: ignoreversion recursesubdirs createallsubdirs
; NOTE: Don't use "Flags: ignoreversion" on any shared system files

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; Tasks: desktopicon

[Run]
; Install TAP driver bundled with OpenConnect
Filename: "{app}\openconnect\tap-setup.exe"; Parameters: "/S"; StatusMsg: "Installing TAP network driver..."; Flags: runascurrentuser waituntilterminated
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent

[Code]
// You can add logic here to detect if OpenConnect is already installed
// or to prompt the user to install it.
