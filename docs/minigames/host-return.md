# Resultados y regreso al sistema solar

El sistema solar y South Park se ejecutan como funciones del mismo proceso. Los dos binarios disponibles son puntos de entrada independientes; el sistema solar no lanza `southpark.exe`. Cada escena libera su ventana, recursos gráficos y audio antes de que se abra la siguiente. No se introduce un gestor global de escenas ni comunicación por archivos de estado.

## Contrato compartido

`src/minigames/common/host.rs` define:

- `MinigameResult::{Victory, Defeat, Abandoned}`.
- `MinigameExit::{Returned(MinigameResult), WindowClosed}`: cerrar explícitamente la ventana no se confunde con abandonar desde pausa.
- `MinigameCompletion { planet_id, minigame_id, result }`: resultado estructurado que el anfitrión retiene en memoria para futuros consumidores de progreso. También se imprime como `MINIGAME_RETURN` en JSON.
- `ReturnConfirmation`: pedir, cancelar y confirmar un abandono; confirmar sin petición pendiente no produce un resultado.

`scenes::minigame_host::run` centraliza el despacho y el estado entre visitas. Actualmente solo registra South Park. Para integrar otro minijuego se agrega su adaptador aquí y se configura el identificador en el planeta. El adaptador devuelve `MinigameExit` y conserva únicamente el contexto entre visitas que corresponda. Minecraft, Smiling Friends, Halloween/Calaca, Regular Show y Star Wars no se implementan ni se modifican en este ajuste.

`SouthParkExit` permanece como alias del contrato compartido para compatibilidad. El único estado de South Park entre visitas es `intro_seen`: no se conserva el progreso del intento. No se entregan tokens, recompensas, desbloqueos ni cinemáticas.

## Menús y controles

- **Victoria:** pantalla con cinco salvadas, Jugar nuevamente y Regresar al sistema solar. Regresar devuelve `Victory`.
- **Derrota:** Reintentar y Regresar al sistema solar. Regresar devuelve `Defeat`.
- **Esc o P:** abre pausa durante exploración y las fases del desafío. Con un mapa o diálogo abierto, Esc cierra primero ese elemento y conserva su prioridad anterior. La intro mantiene Espacio/Enter para saltarla.
- **Pausa:** Continuar partida, Reiniciar minijuego y Regresar al sistema solar. Continuar también acepta Esc, P o Enter.
- **Regresar desde pausa:** abre confirmación; Confirmar/Enter devuelve `Abandoned`, Cancelar/Esc mantiene la partida pausada. Otro Esc o Continuar reanuda.
- **Retroceso en exploración:** conserva el acceso rápido a regresar, ahora con la misma confirmación.
- **Cerrar la ventana o Ctrl+Q:** conserva la salida explícita de la aplicación. En el ejecutable independiente, regresar termina su ejecución porque no existe un anfitrión solar al que volver.

Al pausar se conserva la fase anterior y su reloj. `SaveKennySession::update` retorna antes de mover amenazas, actualizar Kenny, amortiguar su animación o consumir los relojes de búsqueda, activación, muerte y diálogo. La confirmación usa la misma pausa. La interfaz sigue recibiendo entrada y dibujándose. La voz se pausa y se ajusta su reloj de respaldo al continuar; la música usa la pausa existente y los efectos se detienen. El cuadro de reanudación no consume tiempo de gameplay.

Reintentar o reiniciar reconstruye escena y sesión de gameplay, conservando los recursos gráficos cargados y las configuraciones originales. Restablece jugador, Kenny, amenazas, colisiones temporales, objetivos, tres vidas, cero salvadas, relojes, mensajes y orden de amenazas. Comienza un desafío nuevo; no vuelve a reproducir la intro ni reutiliza progreso anterior. No se cambian parámetros de dificultad ni la ventana de reacción de Kenny.

## Contexto del anfitrión

La `Session` solar permanece en memoria mientras transcurre el minijuego: nave seleccionada, órbitas, ángulos, configuración y preferencias no se reconstruyen. Las órbitas quedan detenidas durante la visita.

Al regresar, la ventana solar se abre con la misma nave, velocidad cero, cámara de seguimiento y objetivo en el planeta visitado. La nave queda a 2,2 radios del centro del planeta, en el lado iluminado y mirando hacia afuera, fuera del alcance de exploración y de las colisiones. La cámara vuelve a tomar su posición de seguimiento y se recaptura el cursor. Es necesario soltar E, Enter y el clic antes de habilitar una nueva interacción. El regreso no ejecuta automáticamente otra visita.

## Verificación

```powershell
cargo build --locked --bins
cargo test --locked
cargo run --locked --bin solarsystem -- --smoke-returns
cargo run --locked --bin southpark -- --smoke-session --mute
```

`--smoke-returns` realiza cuatro visitas en un solo proceso: victoria, derrota y dos abandonos. Los dos últimos repiten pausa, congelación del reloj, cancelación, continuación, reinicio y confirmación. Cambia la nave entre visitas y comprueba su conservación, la inmovilidad de las órbitas durante el minijuego, el resultado y la distancia segura al volver. Victoria y derrota se alcanzan con el controlador existente de amenazas, eventos de teclado/mouse y raycasting; no se asignan estados terminales artificialmente. La automatización coloca y orienta al jugador para acercarlo a Kenny y a las interacciones, de modo que no constituye una prueba manual de recorrido por la ciudad.

Las capturas y los informes se guardan en `target/minigame-return-Victory`, `target/minigame-return-Defeat`, `target/minigame-return-Abandoned` y `target/solarsystem`. Así esta prueba no depende de permisos de escritura sobre las referencias históricas de la intro. Cada visita imprime `RETURN_SMOKE_PASS`; la secuencia completa termina con `VISIT_SMOKE_PASS visitas=4`.

`tests/minigame_returns.rs` comprueba el contrato para seis identidades, la confirmación, la congelación de posiciones/animaciones/relojes en once fases y el reinicio completo sin alterar los catorce parámetros de balance. La prueba anterior de sesión ahora usa Reintentar desde derrota y Continuar desde pausa, de acuerdo con los nuevos menús.

Se verificaron mediante automatización la entrada, los tres resultados y el regreso, las reentradas, la confirmación y su cancelación, pausa/continuación y reinicio. `--smoke-session --mute` también completó cinco salvadas, una derrota, Reintentar desde su pantalla y otra pausa/continuación (`SESSION_SMOKE_PASS`). La compilación de todos los binarios y las 110 pruebas pasaron. No se verificó una sesión manual con teclado/mouse físicos ni se evaluaron recompensas o guardado, que quedan fuera del alcance.
