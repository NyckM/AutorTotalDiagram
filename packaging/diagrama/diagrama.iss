; Instalador do Diagrama do Autor Total para Windows (Inno Setup 6).
; Compilado pelo GitHub Actions (.github/workflows/diagrama.yml) ou à mão:
;   cargo build --release -p designcraft --bin diagrama
;   "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" /DVersao=0.20.1 packaging\diagrama\diagrama.iss

#ifndef Versao
  #define Versao "0.1.0"
#endif

[Setup]
AppId={{6E1B7C2A-3D5F-4C8E-9A21-D1A6A0C0B123}
AppName=Diagrama do Autor Total
AppVersion={#Versao}
AppPublisher=Autor Total
AppPublisherURL=https://autortotal.com.br
AppSupportURL=https://autortotal.com.br
DefaultDirName={autopf}\Diagrama do Autor Total
DefaultGroupName=Autor Total
DisableProgramGroupPage=yes
OutputDir=..\..\dist
OutputBaseFilename=DiagramaAutorTotal-Setup
SetupIconFile=..\..\assets\app-icon\designcraft.ico
UninstallDisplayIcon={app}\Diagrama do Autor Total.exe
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequiredOverridesAllowed=dialog
ChangesAssociations=yes
LicenseFile=..\..\LICENSE-MIT

[Languages]
Name: "ptbr"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[Tasks]
Name: "desktopicon"; Description: "Criar um atalho na Área de Trabalho"; GroupDescription: "Atalhos:"

[Files]
Source: "..\..\target\release\diagrama.exe"; DestDir: "{app}"; DestName: "Diagrama do Autor Total.exe"; Flags: ignoreversion
Source: "..\..\LICENSE-MIT"; DestDir: "{app}\licencas"; Flags: ignoreversion
Source: "..\..\LICENSE-APACHE"; DestDir: "{app}\licencas"; Flags: ignoreversion
Source: "..\..\NOTICE"; DestDir: "{app}\licencas"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\Diagrama do Autor Total"; Filename: "{app}\Diagrama do Autor Total.exe"
Name: "{autodesktop}\Diagrama do Autor Total"; Filename: "{app}\Diagrama do Autor Total.exe"; Tasks: desktopicon

[Registry]
; Arquivos .diagramado abrem no Diagrama.
Root: HKA; Subkey: "Software\Classes\.diagramado"; ValueType: string; ValueName: ""; ValueData: "AutorTotal.Diagramado"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\AutorTotal.Diagramado"; ValueType: string; ValueName: ""; ValueData: "Livro diagramado (Autor Total)"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\AutorTotal.Diagramado\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\Diagrama do Autor Total.exe,0"
Root: HKA; Subkey: "Software\Classes\AutorTotal.Diagramado\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\Diagrama do Autor Total.exe"" ""%1"""

[Run]
Filename: "{app}\Diagrama do Autor Total.exe"; Description: "Abrir o Diagrama do Autor Total"; Flags: nowait postinstall skipifsilent
