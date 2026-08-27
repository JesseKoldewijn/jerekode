; NSIS installer for jereko CLI (unsigned pre-1.0)
; Build: makensis /DVERSION=0.0.3 /DBINARY=path/to/jereko.exe /DOUTFILE=out.exe packaging/nsis/jereko.nsi
!include "MUI2.nsh"
!include "WinMessages.nsh"

!ifndef VERSION
!define VERSION "0.0.0"
!endif
!ifndef OUTFILE
!define OUTFILE "jereko-setup.exe"
!endif
!ifndef BINARY
!error "Pass /DBINARY=path/to/jereko.exe"
!endif

Name "jereko ${VERSION}"
OutFile "${OUTFILE}"
InstallDir "$PROGRAMFILES64\jereko"
RequestExecutionLevel admin

!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Section "jereko" SecMain
  SetOutPath "$INSTDIR"
  File /oname=jereko.exe "${BINARY}"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  ; Prepend install dir to machine PATH (native registry; EnVar plugin not bundled in CI)
  ReadRegStr $0 HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path"
  WriteRegExpandStr HKLM "SYSTEM\CurrentControlSet\Control\Session Manager\Environment" "Path" "$INSTDIR;$0"
  SendMessage ${HWND_BROADCAST} ${WM_WININICHANGE} 0 "STR:Environment" /TIMEOUT=5000
SectionEnd

Section "Uninstall"
  Delete "$INSTDIR\jereko.exe"
  Delete "$INSTDIR\Uninstall.exe"
  RMDir "$INSTDIR"
SectionEnd
