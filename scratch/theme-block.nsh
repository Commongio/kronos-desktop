; KRONOS THEME. MUI2 does call SetCtlColors on the two finish-page checkboxes,
; but a checkbox under Windows visual styles paints its own caption and
; ignores it - so "Run KRONOS" came out black on black. Stripping the theme
; fixes the colour but turns the box classic-square. So instead: keep the
; themed box, blank its caption, and lay a white label over where the caption
; was. Clicking the label toggles the box, so it behaves as one control.
!ifndef SS_NOTIFY
  !define SS_NOTIFY 0x0100
!endif
Var KronosLabelRun
Var KronosLabelDesktop

; In: $R0 = checkbox hwnd.  Out: $R1 = the label laid over its caption.
Function KRONOS_Relabel
  System::Call 'user32::GetWindowTextW(p $R0, w .r6, i 256)'
  SendMessage $R0 ${WM_SETTEXT} 0 "STR:"
  ; The checkbox rectangle in the page's own pixels, whatever the DPI.
  System::Call '*(i, i, i, i) p .r1'
  System::Call 'user32::GetWindowRect(p $R0, p r1)'
  System::Call 'user32::MapWindowPoints(p 0, p $mui.FinishPage, p r1, i 2)'
  System::Call '*$1(i .r2, i .r3, i .r4, i .r5)'
  System::Free $1
  ; The box glyph is about as wide as the control is tall. Shrink the
  ; checkbox to exactly that, so it no longer paints over the caption area
  ; (a 195u-wide empty checkbox draws its background on top of anything put
  ; there), then start the label just past it.
  IntOp $7 $5 - $3
  System::Call 'user32::SetWindowPos(p $R0, p 0, i 0, i 0, i $7, i $7, i 0x0006)'
  IntOp $2 $2 + $7
  IntOp $2 $2 + 2
  IntOp $4 $4 - $2
  nsDialogs::CreateControl STATIC ${__NSD_Label_STYLE}|${SS_NOTIFY} 0 $2 $3 $4 $7 "$6"
  Pop $R1
  SetCtlColors $R1 "${MUI_TEXTCOLOR}" "${MUI_BGCOLOR}"
FunctionEnd

Function KRONOS_ToggleRun
  Pop $0
  ${NSD_GetState} $mui.FinishPage.Run $0
  ${If} $0 == ${BST_CHECKED}
    ${NSD_Uncheck} $mui.FinishPage.Run
  ${Else}
    ${NSD_Check} $mui.FinishPage.Run
  ${EndIf}
FunctionEnd

Function KRONOS_ToggleDesktop
  Pop $0
  ${NSD_GetState} $mui.FinishPage.ShowReadme $0
  ${If} $0 == ${BST_CHECKED}
    ${NSD_Uncheck} $mui.FinishPage.ShowReadme
  ${Else}
    ${NSD_Check} $mui.FinishPage.ShowReadme
  ${EndIf}
FunctionEnd

Function KRONOS_FinishShow
  StrCpy $R0 $mui.FinishPage.Run
  Call KRONOS_Relabel
  StrCpy $KronosLabelRun $R1
  ${NSD_OnClick} $KronosLabelRun KRONOS_ToggleRun

  StrCpy $R0 $mui.FinishPage.ShowReadme
  Call KRONOS_Relabel
  StrCpy $KronosLabelDesktop $R1
  ${NSD_OnClick} $KronosLabelDesktop KRONOS_ToggleDesktop
FunctionEnd
