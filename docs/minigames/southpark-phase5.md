# SAVE KENNY! — fase 5

## Ejecutar una partida

```powershell
cargo run
```

Acercarse a Kenny, apuntarle y pulsar **E**. Elegir **Comenzar desafío** o Enter. Cancelar/Escape permite seguir explorando. No se necesita Blender ni numpad para jugar.

La partida empieza con **3 vidas y 0/5 salvadas**. Se barajan las cinco amenazas una sola vez. Superar una avanza el contador; fallarla cuesta una vida después del diálogo y obliga a reintentar la misma. Las superadas no vuelven a aparecer. Cinco éxitos dan victoria; tres muertes completas dan derrota. Ambas pantallas permiten jugar nuevamente o seguir explorando sin cerrar la aplicación.

| Control | Acción |
| --- | --- |
| WASD / mouse / Shift | Caminar / mirar / correr |
| E | Hablar con Kenny o activar el mecanismo visible |
| Enter / clic izquierdo | Elegir una opción del menú |
| Escape | Cancelar menú o liberar/recuperar cursor en el mundo |
| P | Pausar el desafío; continuar o abandonarlo |
| F1 / F2 / F3 / F12 | Ayuda / colisiones / indicador Kenny / captura |
| Ctrl+Q / cerrar ventana | Salir |

**Escape en el mundo no pausa el peligro; P sí lo pausa.** Durante la muerte y las voces, los controles quedan bloqueados hasta finalizar la secuencia. En laptop las teclas F pueden necesitar Fn.

Cada transición reúne al jugador en una posición validada al inicio de North Avenue, mirando hacia el pueblo. El cuadro explica el cambio y presenta la amenaza. El peligro **no empieza mientras se lee**: al confirmar suena la alerta y comienza la preparación de 3 s del director existente. Durante esos segundos se puede caminar y localizar el mecanismo.

## Administrador y reglas

`SaveKennySession` coordina `SouthParkScene` y el `ThreatDirector` existente. No reemplaza el FPS, el raycasting, el controlador de Kenny ni las cinco amenazas. Sus estados son:

```text
Exploring -> menú de Kenny -> Briefing -> Preparing -> Active
Active -> Success -> Briefing de la siguiente amenaza
                  -> Victory al completar 5/5
Active -> DeathAnimation -> DeathDialogue -> Restoring
Restoring -> Briefing del mismo evento si quedan vidas
          -> Defeat después de la tercera muerte
Preparing/Active <-> Paused
Menús y resultados -> Exploring
```

El menú de invitación reutiliza `KennyDialogue`; la aceptación normal delega a la sesión. Su antiguo estado preparatorio se conserva solo para las pruebas históricas de fase 3.

Fisher–Yates y un PRNG local xorshift64* generan la permutación, con semilla del reloj renovada en cada partida. No hay dependencias nuevas ni guardado persistente. El índice pendiente avanza únicamente al confirmar un éxito. El resultado se consume al salir de `Active`, de modo que conservarlo en el director no suma salvadas repetidas.

La muerte conserva su animación original durante 1.5 s. Después reproduce una sola vez la grabación completa de Stan y Kyle. La vida se descuenta al terminar la voz/subtítulos y se entra en `Restoring` antes de limpiar. Si una reaparición está bloqueada, reintentar la limpieza **no vuelve a descontar vidas**. Game Over espera a que finalice el último diálogo.

Los fallos de transición se muestran en el menú, desde donde se puede abandonar. El retorno elimina objetivos y sólidos temporales, restaura a Kenny con las funciones seguras existentes y recupera la exploración e interacción normales.

## HUD y menús

`session_ui.rs` presenta tres corazones mediante primitivas, `Kenny salvado: X/5`, nombre del evento e indicadores específicos de luces, tapa, calefactores, freno y seguro. La parte superior deja libres el centro y los mensajes E. Los cuadros de muerte ocupan aproximadamente el cuarto inferior. Se utilizan las dimensiones reales de la ventana, con mínimo de 640×480.

Las pantallas de victoria y derrota muestran las puntuaciones y botones para reintentar o explorar. Los menús controlan el cursor y bloquean el movimiento; al cerrarlos se recupera la captura sin aplicar el primer delta brusco del mouse.

## Audio y recursos

`src/audio.rs` administra modos de música, efectos, reloj de voz, volúmenes y recursos RAII. El dispositivo se crea antes de los recursos y se descarga después. Continúan Rust 2021, raylib-rs 5.5.1 y serde/serde_json, sin dependencias nuevas.

| Recurso original | Uso | Duración decodificada |
| --- | --- | ---: |
| `assets/audio/music/South Park/exploration.mp3` | Exploración, bucle | 860.395 s |
| `assets/audio/music/South Park/save_kenny.mp3` | Desafío, bucle | 215.719 s |
| `assets/audio/sfx/South Park/danger.mp3` | Preparación | 5.381 s |
| `assets/audio/sfx/South Park/footsteps.mp3` | Movimiento real del FPS | 7.584 s |
| `assets/audio/sfx/South Park/success.mp3` | Amenaza superada | 3.408 s |
| `assets/audio/sfx/South Park/victory.mp3` | Victoria | 3.504 s |
| `assets/audio/sfx/South Park/game_over.mp3` | Derrota tras el diálogo | 3.082 s |
| `assets/audio/sfx/South Park/they_killed_kenny.mp3` | Ambas voces, una reproducción por muerte | 3.343673 s |

`intro.mp3` existe, pero **no se carga ni reproduce**; queda reservado para fase 6. Los ocho archivos de la tabla se cargan una sola vez. Las músicas se pausan/resumen conservando posición. Solicitar el mismo modo por frame no reinicia la pista. Durante muerte, pausa y resultados finales no suena la música; la voz usa un stream sin bucle y los streams activos se actualizan continuamente. Alertas y resultados reemplazan los efectos anteriores para evitar mezclas de fases diferentes.

Los pasos usan un solo `Sound`: se inician con desplazamiento real, no al presionar una tecla contra una pared. No se reinician mientras suenan, se detienen al parar y aumentan de pitch al correr. Menús y reposicionamientos de transición no generan pasos.

Los audios ausentes, errores de carga o falta de dispositivo producen avisos y permiten seguir jugando con subtítulos temporizados. Un dispositivo que deja de avanzar también dispone de respaldo. Los retratos tienen un espacio de reserva si fallan.

Volúmenes en `config/southpark-audio.json`, rango 0–1:

```json
{"music_volume":0.3,"sfx_volume":0.65,"voice_volume":1.0,"footsteps_volume":0.22}
```

Reiniciar para aplicar cambios. `cargo run --bin southpark -- --mute` carga y verifica todo con los volúmenes en cero.

### Stan y Kyle

Se utilizan exactamente `assets/textures/minigames/South Park/StanMarsh.png` y `KyleBrofloski.png`. El rectángulo de dibujo enfatiza el rostro conservando proporciones y transparencia; los PNG no se editan. El cuadro azul inferior tiene borde claro, retrato, nombre y texto progresivo:

- Stan: **Oh my God! They killed Kenny!**
- Kyle: **You bastards!**

`GetMusicTimePlayed` controla texto y cambio de retrato; cambiar al personaje no reinicia el MP3. `IsMusicStreamPlaying` determina su final antes del reintento/Game Over.

`config/southpark-session.json` configura:

| Parámetro | Valor inicial |
| --- | ---: |
| `death_animation_seconds` | 1.5 s |
| `voice_fallback_seconds` | 3.35 s; se sustituye por la duración real si carga |
| `kyle_switch_seconds` | **2.18 s** |
| `success_hold_seconds` | 1.3 s |
| `text_characters_per_second` | 45 |

El análisis decodificó 147456 frames estéreo a 44100 Hz. Se eligió 2.18 s como valor inicial tras una pausa próxima a la última intervención. **No se verificó por escucha ni reconocimiento de voz. Es necesario escuchar la grabación y ajustar ese parámetro si corresponde.** No se afirma sincronización fonética exacta.

Análisis reproducible sin modificar el MP3; Blender solo es necesario para esta herramienta opcional:

```powershell
blender --background --python-exit-code 1 --python tools/audio/inspect_southpark_voice.py
```

El informe queda en `target/phase5-voice-analysis.json`.

## Desarrollo y pruebas

```powershell
cargo run --locked --bin southpark -- --dev-threats
cargo run --locked --bin southpark -- --threat piano
cargo run --locked --bin southpark -- --threat snowman
```

El modo explícito de desarrollo conserva F5 autobús, F6 OVNI, F7 alcantarilla, F10 piano, F11 muñeco, F8 reinicio y F9 limpieza. No consume vidas ni suma salvadas. Estas teclas se ignoran durante una partida normal, que puede completarse mediante los controles habituales y menús.

```powershell
cargo fmt --all -- --check
cargo check --locked
cargo build --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --locked --bin southpark -- --smoke
cargo run --locked --bin southpark -- --smoke-kenny
cargo run --locked --bin southpark -- --smoke-threats
cargo run --locked --bin southpark -- --smoke-session
```

Los tres primeros modos gráficos escriben en `target/southpark-smoke/`, `target/southpark-kenny-smoke/` y `target/southpark-threats-smoke/`. Se preservan las capturas históricas 4A/4B. La prueba nueva genera las referencias de fase 5 y respeta el reloj de audio real, limitándose a 240 FPS para permitir que termine la voz.

Pasaron **71 pruebas**: 19 FPS, 15 Kenny, 23 amenazas y 14 de sesión/audio. Incluyen permutaciones, conteo único, conservación de amenaza pendiente, victoria, derrota, pausa, limpieza, reaparición bloqueada y audio ausente. También pasaron compilación, formato y clippy sin advertencias.

La prueba gráfica normal usó renderizado real, eventos de entrada de raylib y el raycasting compartido. Completó cinco éxitos diferentes; confirmó otra partida; volvió a explorar y hablar con Kenny; falló tres veces el mismo evento pendiente; mostró derrota; volvió a explorar y comprobó abandono desde pausa. Validó ocho audios, dos retratos, cinco efectos de éxito, tres voces y un efecto de victoria/Game Over. Pasaron también las regresiones gráficas de FPS, diálogo y diez resultados de las amenazas individuales.

La regresión FPS registró una reproducción de pasos durante su tramo de movimiento, sin reinicios por frame. La automatización posiciona y orienta al jugador; **no reemplaza una partida manual ni una evaluación de dificultad, audibilidad, mezcla o sincronización de voces**. Queda revisar teclado/mouse físicos y escuchar la grabación.

Los SHA-256 de los 87 recursos protegidos del estado inicial se compararon antes/después: planeta, ciudad, naves, amenazas, PNG y MP3 intactos. Los tres `.gitkeep` eliminados y los MP3/retratos sin seguimiento ya estaban así al iniciar el trabajo; se preservó ese estado.

### Referencias

- [HUD](../references/southpark/phase5/hud.png), [vidas después de morir](../references/southpark/phase5/lives.png).
- [Stan](../references/southpark/phase5/stan.png), [Kyle](../references/southpark/phase5/kyle.png).
- [Victoria](../references/southpark/phase5/victory.png), [derrota](../references/southpark/phase5/defeat.png).
- [Preparación](../references/southpark/phase5/briefing.png), [pausa](../references/southpark/phase5/pause.png).
- [Validación de partida/audio](../references/southpark/phase5/runtime-validation.json).
- [Regresiones y recursos originales](../references/southpark/phase5/regression-validation.json).

## Archivos y fase 6

Modificados: `README.md`, `src/lib.rs`, `src/minigames/southpark/mod.rs`, `src/scenes/mod.rs`, `src/scenes/kenny_dialogue.rs`, `src/scenes/southpark_dev.rs`.

Creados: `src/audio.rs`, `src/minigames/southpark/session.rs`, `src/scenes/session_ui.rs`, `src/scenes/session_smoke.rs`, `tests/save_kenny_session.rs`, `config/southpark-audio.json`, `config/southpark-session.json`, `tools/audio/inspect_southpark_voice.py`, este documento y ocho capturas/dos informes en `docs/references/southpark/phase5/`. Logs y análisis auxiliares quedan en `target/`, ignorado por Git. No cambiaron dependencias, modelos ni los MP3/PNG originales.

Fase 6 puede conectar entrada/salida del planeta con la sesión y sus resultados terminales. Quedan pendientes la presentación cinematográfica, `intro.mp3` y la integración solar. No se agregó guardado persistente ni menú global. No hubo commit, push, merge, rebase ni cambio de rama.
