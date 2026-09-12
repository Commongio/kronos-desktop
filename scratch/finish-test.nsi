; scratch/finish-test.nsi — just the finish page, for iterating on the theme
; block without a ten-minute CI round trip. Builds with Tauri's own makensis.
Unicode true
ManifestDPIAware true
!include MUI2.nsh
!include nsDialogs.nsh
Name "KRONOS finish test"
OutFile "finish-test.exe"
RequestExecutionLevel user
InstallDir "$TEMP\kronos-finish-test"
!define MUI_ICON "..\src-tauri\icons\icon.ico"
!define MUI_WELCOMEFINISHPAGE_BITMAP "..\src-tauri\installer\nsis-sidebar.bmp"
!define MUI_BGCOLOR "000000"
!define MUI_TEXTCOLOR "FFFFFF"
!define MUI_FINISHPAGE_TITLE "KRONOS is installed"
!define MUI_FINISHPAGE_TITLE_3LINES
!define MUI_FINISHPAGE_TEXT "Sign in with your Terminal account.$\r$\n$\r$\nIf the desktop shortcut shows a plain icon, it becomes the KT mark the first time you launch the app."
!define MUI_FINISHPAGE_SHOWREADME
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Create desktop shortcut"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION Nop
!define MUI_FINISHPAGE_RUN
!define MUI_FINISHPAGE_RUN_TEXT "Run KRONOS"
!define MUI_FINISHPAGE_RUN_FUNCTION Nop
!define MUI_PAGE_CUSTOMFUNCTION_SHOW KRONOS_FinishShow
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_LANGUAGE "English"
Function Nop
FunctionEnd
Section
SectionEnd
!include "theme-block.nsh"
