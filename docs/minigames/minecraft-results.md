# Diamond Rush: resultados independientes de la navegación

## Causa

El menú de aldea abría siempre `ConfirmReturn` y preguntaba si se quería abandonar
Minecraft, sin comprobar si existía una partida activa. Además, `visit_result`
era una variable local que se asignaba al volver de la mina, no un registro del
evento de victoria. Esto confundía navegación con resultado y no conservaba un
historial verificable entre visitas.

## Flujo corregido

- **Exploración libre:** Escape → Regresar al sistema solar sale directamente.
  Si no hubo partidas en esa visita devuelve `ReturnedWithoutResult`; no se
  registra un abandono ni se vuelve a informar una victoria de una visita anterior.
- **Partida activa:** Escape mantiene la pausa existente. Elegir Aldea abre
  “Abandonar Diamond Rush” con “Perderás el progreso de la partida actual”.
  Cancelar/Escape reanuda el mismo intento con su tiempo y diamantes; confirmar
  finaliza exclusivamente ese intento como `Abandoned` y vuelve a la aldea.
- **Victoria/derrota:** las reglas existentes emiten `RushCue::Victory` o
  `RushCue::GameOver`. En ese mismo frame el host registra `Victory` o `Defeat`
  y desmarca el intento activo. Las pantallas existentes no se rediseñan.
  Regresar a la aldea y luego al espacio no cambia ese resultado ni solicita
  confirmación de abandono.
- **Reintento:** utiliza el reinicio existente de bloques, colisiones, diamantes,
  reloj y peligros. Recibe un nuevo ID; la victoria anterior permanece en el
  historial, pero no convierte al nuevo intento en victoria. Reiniciar una partida
  inconclusa finaliza la anterior como abandonada.
- **Cierre/error:** el adaptador finaliza como abandonado solo un intento todavía
  activo; jamás sobrescribe uno ya completado.

La cantidad de diamantes, los 150 segundos, la lava, raycasting, minerales,
modelos, música, SFX y reglas no se modificaron. `RushState::Victory/Defeat`
continúa deteniendo reloj y peligros mediante las condiciones existentes.
No se modificó el comportamiento de South Park ni otros minijuegos.

## Sesión y futuro punto de recompensas

`MinecraftVisits` vive en `MinigameVisits.minecraft`, dentro de la sesión solar.
Mantiene ID incremental, intento activo, historial `CompletedAttempt` y eventos
`AttemptEvent::{Started, Finished}`. El identificador es único durante esa sesión.
Solo un intento activo puede finalizar y `active.take()` impide registrar dos
veces el mismo resultado aunque llegue otra notificación terminal.

`record_cues` recibe los resultados explícitos de las reglas. Ni abrir un menú,
ni cambiar de escena, ni salir de Minecraft generan una victoria. Recoger ocho
diamantes tampoco basta: la regla de victoria sigue exigiendo la salida real
con tiempo positivo. `exit_since` únicamente consulta resultados de la visita;
no agrega eventos ni modifica el historial.

Un consumidor futuro puede usar `drain_events()` y filtrar
`AttemptEvent::Finished(CompletedAttempt { result: Victory, attempt_id })`.
Cada evento se entrega una vez y el historial se conserva después de consumirlo.
Ese es el punto para recompensas futuras; `MINIGAME_RETURN` sigue siendo metadata
de navegación y no debe tratarse como otro evento de recompensa.

**No se otorgan tokens ni se guardan datos en disco.** El registro dura lo que
dura la sesión del sistema solar. Cerrar la aplicación lo descarta; no hay un
sistema de persistencia nuevo ni garantías de consumo duradero entre procesos.

## Archivos de esta corrección

- `src/minigames/minecraft/visit.rs`: historial/eventos y menú de aldea sin advertencia.
- `src/scenes/minecraft_rush.rs`: registro al recibir el resultado, confirmación
  exclusiva de abandono activo y devolución del resultado al anfitrión.
- `src/scenes/minigame_host.rs`: conservación de `MinecraftVisits` entre visitas.
- `src/minigames/common/host.rs`: salida de navegación sin resultado de desafío.
- `src/scenes/minecraft_rush_smoke.rs`: casos gráficos de victoria, cancelación
  y confirmación de abandono.
- `tests/minecraft_results.rs`: cinco pruebas de resultados, reintentos y eventos únicos.
- `tests/minecraft_host.rs`: salida directa de aldea.
- Este documento y actualización de las guías de integración/contrato.

## Verificación

```powershell
cargo check
cargo build
cargo test
cargo run --bin solarsystem -- --smoke-minecraft
cargo run --bin minecraft -- --smoke-rush --capture-dir target/minecraft-outcome-abandon
cargo run --bin minecraft -- --smoke-exploration-exit --capture-dir target/minecraft-outcome-exploration
git diff --check
git diff
git status
```

La automatización solar completó dos victorias con minería real, regreso por la
salida, pantalla de victoria y aldea → espacio sin advertencia. El primer intento
produjo `attempt_id: 1`, el segundo `attempt_id: 2`, ambos `Victory`; el total
persistió entre visitas. No se produjo ningún `Abandoned` en ese recorrido.
La prueba de exploración libre retornó `ReturnedWithoutResult`, cero intentos y
música detenida. La prueba completa de Rush produjo exactamente `Victory`,
`Defeat` y `Abandoned` para tres intentos distintos; comprobó que cancelar el
abandono reanuda el juego antes de confirmar una segunda solicitud.

Se inspeccionaron capturas de victoria, pausa de aldea y advertencia de abandono
activo. Las pruebas usan OpenGL/audio reales con volumen cero, eventos nativos y
posiciones/tiempos prefijados; no son una partida manual con hardware físico.

Pendiente manual: realizar esos tres recorridos sin flags de prueba, volver a
entrar y repetir una partida después de ganar. Escuchar audio y comprobar la
captura del mouse con controles físicos. La ruta de victoria debe imprimir una
sola línea `MINECRAFT_ATTEMPT_FINISHED` para ese ID, con `Victory`, y el regreso
debe conservar ese resultado. En la aldea no debe aparecer ningún mensaje de
abandono. En la mina, Aldea durante pausa sí debe advertir sobre el progreso.

Se mantiene `feature/easter-egg`, con la integración pendiente intacta, sin commit
ni push.
