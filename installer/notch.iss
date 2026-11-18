
#ifndef AppVersion
  #error Pass the version with /DAppVersion=x.y.z
#endif
#ifndef ExeName
  #error Pass the name of the executable with /DExeName=name.exe
#endif
#ifndef SourceExe
  #error Pass the built executable with /DSourceExe=path
#endif
#ifndef SettingsExeName
  #error Pass the name of the settings executable with /DSettingsExeName=name.exe
#endif
#ifndef SourceSettingsExe
  #error Pass the built settings executable with /DSourceSettingsExe=path
#endif

; A version with a suffix (0.5.0-alpha.1) is a dev build: it gets the dev tile, as the executables do, and installs beside
; every other copy instead of replacing one. A stable version replaces the stable copy that is installed.
#if Pos("-", AppVersion) > 0
  #define IsDevBuild
  #define TileName "tile-dev"
  #define AppDisplayName "Foldline " + AppVersion
#else
  #define TileName "tile-stable"
  #define AppDisplayName "Foldline"
#endif

; Keep this ID stable so upgrades find the existing stable installation.
#define AppGuid "{B4D1E7A2-5C3F-4F68-9A0E-2D7C61B8E4F3}"

[Setup]
#ifdef IsDevBuild
; One identity per dev version: its own folder, Start menu entry and uninstall entry.
AppId=B4D1E7A2-5C3F-4F68-9A0E-2D7C61B8E4F3-dev-{#AppVersion}
AppVerName={#AppDisplayName}
DefaultDirName={localappdata}\Programs\{#AppDisplayName}
#else
AppId={{B4D1E7A2-5C3F-4F68-9A0E-2D7C61B8E4F3}
DefaultDirName={localappdata}\Programs\Notch
#endif
AppName={#AppDisplayName}
AppVersion={#AppVersion}
AppPublisher=hibatillah
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
LicenseFile=..\LICENSE
SetupIconFile=..\assets\branding\{#TileName}.ico
WizardSmallImageFile=wizard\{#TileName}-55.png,wizard\{#TileName}-64.png,wizard\{#TileName}-72.png,wizard\{#TileName}-80.png,wizard\{#TileName}-89.png,wizard\{#TileName}-97.png,wizard\{#TileName}-106.png,wizard\{#TileName}-114.png,wizard\{#TileName}-123.png
UninstallDisplayIcon={app}\{#ExeName}
CloseApplications=force
RestartApplications=no

#ifndef IsDevBuild
[Tasks]
Name: "startup"; Description: "Start Foldline when Windows starts"; GroupDescription: "Options:"
#endif

[Files]
Source: "{#SourceExe}"; DestDir: "{app}"; DestName: "{#ExeName}"; Flags: ignoreversion
; The tray menu starts the settings program from the folder of the notch.
Source: "{#SourceSettingsExe}"; DestDir: "{app}"; DestName: "{#SettingsExeName}"; Flags: ignoreversion
Source: "..\THIRD-PARTY-LICENSES.txt"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\LICENSE"; DestDir: "{app}"; DestName: "LICENSE.txt"; Flags: ignoreversion
Source: "..\assets\fonts\Geist-LICENSE.txt"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\{#AppDisplayName}"; Filename: "{app}\{#ExeName}"; IconFilename: "{app}\{#ExeName}"; IconIndex: 0

#ifndef IsDevBuild
[Registry]
; The same entry the "Start with Windows" switch on the settings window's General page sets; a dev build never writes it.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "Notch"; ValueData: """{app}\{#ExeName}"""; Flags: uninsdeletevalue; Tasks: startup
#endif

[Run]
Filename: "{app}\{#ExeName}"; Description: "Start {#AppDisplayName}"; Flags: nowait postinstall skipifsilent

#ifndef IsDevBuild
[Code]
const
  UninstallKey = 'Software\Microsoft\Windows\CurrentVersion\Uninstall\{#AppGuid}_is1';

{ Asks before replacing the stable copy that is already installed, saying which versions are involved. A dev build never asks: it installs beside the others. }
function InitializeSetup(): Boolean;
var
  Installed: String;
  InstalledVersion, NewVersion: Int64;
  Question: String;
begin
  Result := True;
  if not RegQueryStringValue(HKCU, UninstallKey, 'DisplayVersion', Installed) then
    Exit;
  if StrToVersion(Installed, InstalledVersion) and StrToVersion('{#AppVersion}', NewVersion) then
  begin
    if ComparePackedVersion(InstalledVersion, NewVersion) < 0 then
      Question := 'Foldline ' + Installed + ' is already installed.' + #13#10 + 'Update it to version {#AppVersion}?'
    else if ComparePackedVersion(InstalledVersion, NewVersion) = 0 then
      Question := 'Foldline ' + Installed + ' is already installed.' + #13#10 + 'Install it again?'
    else
      Question := 'A newer Foldline (' + Installed + ') is already installed.' + #13#10 + 'Replace it with version {#AppVersion}?';
  end
  else
    Question := 'Foldline ' + Installed + ' is already installed.' + #13#10 + 'Replace it with version {#AppVersion}?';
  Result := SuppressibleMsgBox(Question, mbConfirmation, MB_YESNO, IDYES) = IDYES;
end;
#endif

[Code]
{ Lines the page description up with the page title: the modern wizard style indents it, which looks detached in the header. }
procedure InitializeWizard();
begin
  WizardForm.PageDescriptionLabel.Left := WizardForm.PageNameLabel.Left;
  WizardForm.PageDescriptionLabel.Width := WizardForm.PageNameLabel.Width;
end;
