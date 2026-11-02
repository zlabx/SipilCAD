; Inno Setup script for SipilCAD (Windows desktop build).
;
; This is the installer the SipilStock Apps launcher installs and updates (silently, per user:
; /VERYSILENT /CURRENTUSER, no administrator rights). The MSI next to it (main.wxs) stays for
; machine-wide, managed deployments; both share the same program, icons and registry names.
;
; Build (done by the "SipilCAD desktop (Windows)" workflow in zlabx/Apps, step "Inno installer"):
;     ISCC /DMyAppVersion=2027.0 /DMyAppVersionInfo=2027.0.0.0 /DDistDir=<dir> /DOutDir=<dir> sipilcad.iss
; <dir> holds SipilCAD.exe, dwg_thumbnailer_win.dll, AppIcon.ico, dwg.ico, dxf.ico and docs\*.
;
; The AppId is a FIXED GUID. The launcher finds the installed app through the uninstall key
; {AppId}_is1 (crates/core/src/catalog.rs in zlabx/Launcher: inno_app_id). Changing it breaks
; detection and upgrades; keep both in sync (rebrand/opencadstudio/test checks the value).

#define MyAppName "SipilCAD"
#define MyAppPublisher "SipilStock"
#define MyAppURL "https://sipilstock.com/sipilcad/"
#define MyAppExeName "SipilCAD.exe"

#ifndef MyAppVersion
  #define MyAppVersion "0.0.0"
#endif
#ifndef MyAppVersionInfo
  #define MyAppVersionInfo "0.0.0.0"
#endif
#ifndef DistDir
  #define DistDir "..\..\..\dist"
#endif
#ifndef OutDir
  #define OutDir DistDir
#endif

[Setup]
AppId={{33B8CECF-E398-5B41-9860-58D73AAEA433}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL=https://github.com/zlabx/SipilCAD/issues
AppUpdatesURL={#MyAppURL}
VersionInfoVersion={#MyAppVersionInfo}
VersionInfoCompany={#MyAppPublisher}
VersionInfoDescription=SipilCAD setup
VersionInfoProductName={#MyAppName}

; Per user by default ({autopf} is %LOCALAPPDATA%\Programs then). The dialog lets someone who
; double-clicks the installer choose "all users" (UAC); the launcher always passes /CURRENTUSER.
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog

; Free software: the license page shows the GPL-3.0 text.
LicenseFile={#DistDir}\docs\LICENSE.txt

OutputDir={#OutDir}
OutputBaseFilename=sipilcad-setup-v{#MyAppVersion}
SetupIconFile={#DistDir}\AppIcon.ico
UninstallDisplayIcon={app}\{#MyAppExeName}
WizardStyle=modern
; The app registers file types (HKA\Software\Classes): tell Explorer to refresh icons.
ChangesAssociations=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Compression=lzma2
SolidCompression=yes
; An update runs while the previous version may be open: close it, do not restart it.
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
; Not ticked by default: SipilCAD does not take over .dwg/.dxf from the program the user already
; has. It always appears in "Open with" and in Settings > Default apps; the app itself offers to
; become the default on first start.
Name: "assocdwg"; Description: "Open .dwg files with SipilCAD (double-click)"; GroupDescription: "File associations:"; Flags: unchecked
Name: "assocdxf"; Description: "Open .dxf files with SipilCAD (double-click)"; GroupDescription: "File associations:"; Flags: unchecked

[Files]
Source: "{#DistDir}\SipilCAD.exe"; DestDir: "{app}"; Flags: ignoreversion
; Explorer thumbnails for DWG; the app registers it per user when asked.
Source: "{#DistDir}\dwg_thumbnailer_win.dll"; DestDir: "{app}"; Flags: ignoreversion
; Branded document icons, referenced by the ProgIds below and by the app's own registration.
Source: "{#DistDir}\dwg.ico"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#DistDir}\dxf.ico"; DestDir: "{app}"; Flags: ignoreversion
; The About dialog opens these from the folder of the exe.
Source: "{#DistDir}\docs\LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#DistDir}\docs\SOURCE.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#DistDir}\docs\THIRD_PARTY_LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#DistDir}\docs\OFL-PTSans.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon

[Registry]
; Same names as the MSI (main.wxs) and as the app's own per-user registration (src/io/file_association.rs,
; patch 0014): Applications\SipilCAD.exe, ProgIds SipilCAD.DWG / SipilCAD.DXF, Software\SipilCAD\Capabilities.
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\DefaultIcon"; ValueType: string; ValueData: "{app}\{#MyAppExeName},0"
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\shell\open\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\SupportedTypes"; ValueType: string; ValueName: ".dwg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\{#MyAppExeName}\SupportedTypes"; ValueType: string; ValueName: ".dxf"; ValueData: ""

Root: HKA; Subkey: "Software\Classes\SipilCAD.DWG"; ValueType: string; ValueData: "DWG Drawing"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\SipilCAD.DWG\DefaultIcon"; ValueType: string; ValueData: "{app}\dwg.ico,0"
Root: HKA; Subkey: "Software\Classes\SipilCAD.DWG\shell\open\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""
Root: HKA; Subkey: "Software\Classes\SipilCAD.DXF"; ValueType: string; ValueData: "DXF Drawing"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\SipilCAD.DXF\DefaultIcon"; ValueType: string; ValueData: "{app}\dxf.ico,0"
Root: HKA; Subkey: "Software\Classes\SipilCAD.DXF\shell\open\command"; ValueType: string; ValueData: """{app}\{#MyAppExeName}"" ""%1"""

; "Open with": SipilCAD is offered without becoming the default.
Root: HKA; Subkey: "Software\Classes\.dwg\OpenWithProgids"; ValueType: string; ValueName: "SipilCAD.DWG"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.dxf\OpenWithProgids"; ValueType: string; ValueName: "SipilCAD.DXF"; ValueData: ""; Flags: uninsdeletevalue
; Optional tasks: become the default opener. Uninstall removes the value; Windows falls back.
Root: HKA; Subkey: "Software\Classes\.dwg"; ValueType: string; ValueData: "SipilCAD.DWG"; Tasks: assocdwg; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.dxf"; ValueType: string; ValueData: "SipilCAD.DXF"; Tasks: assocdxf; Flags: uninsdeletevalue

; The parent key holds Capabilities (below) and whatever the app registers per user at run time; remove it on
; uninstall only when nothing is left in it.
Root: HKA; Subkey: "Software\SipilCAD"; Flags: uninsdeletekeyifempty

; Settings > Default apps.
Root: HKA; Subkey: "Software\SipilCAD\Capabilities"; ValueType: string; ValueName: "ApplicationName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\SipilCAD\Capabilities"; ValueType: string; ValueName: "ApplicationDescription"; ValueData: "2D/3D CAD application for DWG and DXF drawings"
Root: HKA; Subkey: "Software\SipilCAD\Capabilities\FileAssociations"; ValueType: string; ValueName: ".dwg"; ValueData: "SipilCAD.DWG"
Root: HKA; Subkey: "Software\SipilCAD\Capabilities\FileAssociations"; ValueType: string; ValueName: ".dxf"; ValueData: "SipilCAD.DXF"
Root: HKA; Subkey: "Software\RegisteredApplications"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: "Software\SipilCAD\Capabilities"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#MyAppName}}"; Flags: nowait postinstall skipifsilent
