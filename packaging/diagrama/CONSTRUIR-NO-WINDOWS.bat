@echo off
setlocal
chcp 65001 >nul
cd /d "%~dp0\..\.."
echo ============================================================
echo   Diagrama do Autor Total - gerar o aplicativo no Windows
echo ============================================================
echo.
where cargo >nul 2>nul
if errorlevel 1 (
  echo [1/3] Instalando o Rust e as ferramentas de compilacao da Microsoft...
  echo       ^(so na primeira vez; pode levar de 10 a 30 minutos^)
  winget install -e --id Rustlang.Rustup --accept-package-agreements --accept-source-agreements
  winget install -e --id Microsoft.VisualStudio.2022.BuildTools --override "--quiet --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended" --accept-package-agreements --accept-source-agreements
  echo.
  echo  Feche esta janela e rode este arquivo de novo ^(para o Windows achar o Rust^).
  pause
  exit /b 0
)
echo [2/3] Compilando ^(a primeira vez demora; as seguintes sao rapidas^)...
cargo build --release -p designcraft --bin diagrama
if errorlevel 1 (
  echo [ERRO] A compilacao falhou. Veja as mensagens acima.
  pause
  exit /b 1
)
if not exist dist mkdir dist
copy /y target\release\diagrama.exe "dist\Diagrama do Autor Total.exe" >nul
echo [3/3] Gerando o instalador ^(se o Inno Setup estiver instalado^)...
if exist "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" (
  "%ProgramFiles(x86)%\Inno Setup 6\ISCC.exe" packaging\diagrama\diagrama.iss
) else (
  echo  Inno Setup nao encontrado: so o .exe portatil foi gerado.
  echo  Para o instalador: winget install -e --id JRSoftware.InnoSetup e rode de novo.
)
echo.
echo  PRONTO! Os arquivos estao na pasta dist.
start "" "%cd%\dist"
pause
