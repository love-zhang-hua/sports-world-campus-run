Unicode true
!include "MUI2.nsh"
!include "FileFunc.nsh"
Name "运动路线助手"
OutFile "dist\SportsRoute-1.7.0-Setup.exe"
InstallDir "$LOCALAPPDATA\Programs\SportsRoute"
RequestExecutionLevel user
SetCompressor /SOLID lzma
BrandingText "运动路线助手 · Windows 1.7.0"
VIProductVersion "1.7.0.0"
VIAddVersionKey /LANG=2052 "ProductName" "运动路线助手"
VIAddVersionKey /LANG=2052 "FileDescription" "运动路线助手安装程序"
VIAddVersionKey /LANG=2052 "FileVersion" "1.7.0"
VIAddVersionKey /LANG=2052 "LegalCopyright" "Based on YanamiNeko / Sports-simulation; CC BY-NC 4.0"
Icon "assets\app.ico"
UninstallIcon "assets\app.ico"
!define MUI_ABORTWARNING
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "SimpChinese"
Var Smoke
Var Params

Function .onInit
  SetShellVarContext current
  ${GetParameters} $Params
  ClearErrors
  ${GetOptions} $Params "/VERIFYINSTALL" $Smoke
  IfErrors normal
    StrCpy $Smoke "1"
    Return
  normal:
    StrCpy $Smoke "0"
FunctionEnd

Section "运动路线助手" Main
  SetOutPath "$INSTDIR"
  File "dist\SportsRoute.exe"
  File "LICENSE"
  File "THIRD_PARTY_NOTICES.md"
  File /oname=使用说明.md "docs\WINDOWS.md"
  WriteUninstaller "$INSTDIR\Uninstall.exe"
  StrCmp $Smoke "1" smoke normal
  smoke:
    FileOpen $0 "$INSTDIR\installer-verification.marker" w
    FileWrite $0 "Temporary installer verification"
    FileClose $0
    CreateDirectory "$INSTDIR\verification-shortcuts"
    CreateShortCut "$INSTDIR\verification-shortcuts\运动路线助手.lnk" "$INSTDIR\SportsRoute.exe"
    Goto end
  normal:
    CreateDirectory "$SMPROGRAMS\运动路线助手"
    CreateShortCut "$SMPROGRAMS\运动路线助手\运动路线助手.lnk" "$INSTDIR\SportsRoute.exe"
    CreateShortCut "$SMPROGRAMS\运动路线助手\卸载.lnk" "$INSTDIR\Uninstall.exe"
    CreateShortCut "$DESKTOP\运动路线助手.lnk" "$INSTDIR\SportsRoute.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute" "DisplayName" "运动路线助手"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute" "DisplayVersion" "1.7.0"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute" "DisplayIcon" "$INSTDIR\SportsRoute.exe"
    WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute" "InstallLocation" "$INSTDIR"
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute" "NoModify" 1
    WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute" "NoRepair" 1
  end:
SectionEnd

Section "Uninstall"
  SetShellVarContext current
  IfFileExists "$INSTDIR\installer-verification.marker" smoke normal
  normal:
    Delete "$DESKTOP\运动路线助手.lnk"
    Delete "$SMPROGRAMS\运动路线助手\运动路线助手.lnk"
    Delete "$SMPROGRAMS\运动路线助手\卸载.lnk"
    RMDir "$SMPROGRAMS\运动路线助手"
    DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\SportsRoute"
    Goto files
  smoke:
    Delete "$INSTDIR\verification-shortcuts\运动路线助手.lnk"
    RMDir "$INSTDIR\verification-shortcuts"
    Delete "$INSTDIR\installer-verification.marker"
  files:
    Delete "$INSTDIR\SportsRoute.exe"
    Delete "$INSTDIR\LICENSE"
    Delete "$INSTDIR\THIRD_PARTY_NOTICES.md"
    Delete "$INSTDIR\使用说明.md"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir "$INSTDIR"
SectionEnd
