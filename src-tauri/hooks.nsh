; Puerta de desinstalación: antes de borrar nada, ejecuta Asegurao en modo
; --uninstall-gate, que exige la contraseña maestra. Si devuelve un código
; distinto de 0 (contraseña incorrecta o cancelado), se aborta la desinstalación.

!include "LogicLib.nsh"

!macro NSIS_HOOK_PREUNINSTALL
  nsExec::Exec '"$INSTDIR\Asegurao.exe" --uninstall-gate'
  Pop $0
  ${If} $0 != "0"
    MessageBox MB_ICONSTOP|MB_OK "Contraseña incorrecta o cancelada.$\r$\nNo se ha desinstalado Asegurao."
    Abort
  ${EndIf}
!macroend
