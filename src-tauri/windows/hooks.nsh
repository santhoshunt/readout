; Readout installer hooks: Explorer right-click entries and the user PATH entry.
; Everything lives under HKCU, matching installMode "currentUser".

!include "WordFunc.nsh"

!macro READOUT_VERB KEY LABEL ARG
  WriteRegStr HKCU "Software\Classes\${KEY}\shell\Readout" "MUIVerb" "${LABEL}"
  WriteRegStr HKCU "Software\Classes\${KEY}\shell\Readout" "Icon" "$INSTDIR\readout.exe"
  WriteRegStr HKCU "Software\Classes\${KEY}\shell\Readout\command" "" '"$INSTDIR\readout.exe" "${ARG}"'
!macroend

; Reads the user Path into $R0. $R1 is "ok" only when the value was read in full.
; Standard NSIS strings stop at 1024 characters, so a longer Path reads back empty
; and must never be written back.
!macro READOUT_READ_PATH
  StrCpy $R1 "ok"
  ClearErrors
  ReadRegStr $R0 HKCU "Environment" "Path"
  ${If} ${Errors}
    ; A missing value is fine (first entry); any other read error is not.
    ReadRegDWORD $R2 HKCU "Environment" "Path"
    ${IfNot} ${Errors}
      StrCpy $R1 "bad"
    ${EndIf}
    StrCpy $R0 ""
  ${EndIf}
  ${If} $R0 == ""
    ; Empty but present means the read was truncated.
    StrCpy $R3 0
    readout_enum_loop:
      ClearErrors
      EnumRegValue $R4 HKCU "Environment" $R3
      IfErrors readout_enum_done
      ${If} $R4 == "Path"
        StrCpy $R1 "bad"
      ${EndIf}
      IntOp $R3 $R3 + 1
      Goto readout_enum_loop
    readout_enum_done:
  ${EndIf}
  StrLen $R2 $R0
  ${If} $R2 >= 1000
    StrCpy $R1 "bad"
  ${EndIf}
!macroend

!macro NSIS_HOOK_POSTINSTALL
  !insertmacro READOUT_VERB "Directory" "Open folder in Readout" "%V"
  !insertmacro READOUT_VERB "Directory\Background" "Open folder in Readout" "%V"
  !insertmacro READOUT_VERB "SystemFileAssociations\.md" "Open in Readout" "%1"
  !insertmacro READOUT_VERB "SystemFileAssociations\.markdown" "Open in Readout" "%1"
  ; Lifts Explorer's 15-item limit so a large multi-select opens every file as a tab.
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\.md\shell\Readout" "MultiSelectModel" "Player"
  WriteRegStr HKCU "Software\Classes\SystemFileAssociations\.markdown\shell\Readout" "MultiSelectModel" "Player"

  !insertmacro READOUT_READ_PATH
  ${If} $R1 == "ok"
    ${WordFind} "$R0" "$INSTDIR" "E+1{" $R5
    ${If} ${Errors}
      ${If} $R0 == ""
        WriteRegExpandStr HKCU "Environment" "Path" "$INSTDIR"
      ${Else}
        WriteRegExpandStr HKCU "Environment" "Path" "$R0;$INSTDIR"
      ${EndIf}
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
    ${EndIf}
  ${Else}
    FileOpen $R6 "$INSTDIR\ADD-TO-PATH.txt" w
    FileWrite $R6 "$INSTDIR"
    FileClose $R6
    MessageBox MB_OK|MB_ICONINFORMATION "Readout could not add itself to your PATH automatically. To use 'readout .' in a terminal, add this folder to your user PATH:$\r$\n$\r$\n$INSTDIR"
  ${EndIf}

  ; Tells Explorer the file types and icons changed, so an update drops the cached old icon.
  ; Tauri's FileAssociation.nsh defines this macro but its installer never calls it.
  !insertmacro UPDATEFILEASSOC
!macroend

!macro NSIS_HOOK_PREUNINSTALL
  DeleteRegKey HKCU "Software\Classes\Directory\shell\Readout"
  DeleteRegKey HKCU "Software\Classes\Directory\Background\shell\Readout"
  DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.md\shell\Readout"
  DeleteRegKey HKCU "Software\Classes\SystemFileAssociations\.markdown\shell\Readout"

  !insertmacro READOUT_READ_PATH
  ${If} $R1 == "ok"
  ${AndIf} $R0 != ""
    ${WordReplace} "$R0" ";$INSTDIR" "" "+" $R5
    ${WordReplace} "$R5" "$INSTDIR;" "" "+" $R5
    ${If} $R5 == "$INSTDIR"
      StrCpy $R5 ""
    ${EndIf}
    ${If} $R5 != $R0
      WriteRegExpandStr HKCU "Environment" "Path" "$R5"
      SendMessage ${HWND_BROADCAST} ${WM_SETTINGCHANGE} 0 "STR:Environment" /TIMEOUT=5000
    ${EndIf}
  ${EndIf}
!macroend
