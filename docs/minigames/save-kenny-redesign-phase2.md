# Save Kenny 2.0 — fase 2

La partida normal conserva la exploración 3D y abre un puzzle 2D al terminar cada conversación. No hay que correr hacia el peligro ni reconstruir el escenario. La ciudad, cámara, colisiones, NPC, minimapa y recompensas conservan su estado.

## Pantallas y mecánicas

| Encuentro | Puzzle implementado | Victoria / derrota |
| --- | --- | --- |
| Tweek y Craig | Circuito giratorio 4×4, fuente + y tres calefactores. Propagación por puertos recíprocos; acepta cualquier conexión válida. | Conectar los tres / agotar 30 s. |
| Tolkien | Seis luces en un OVNI, selección pseudoaleatoria sin repetir inmediatamente la anterior, ventanas de reacción de 1,55 a 0,85 s. | Resistir 20 s / carga del rayo al 100 %. Los aciertos descargan y los fallos aumentan la carga. |
| Clyde | Cuatro palancas con números y colores. Tres pistas; la solución única se verifica enumerando las 24 permutaciones. | Orden correcto / tres intentos fallidos. |
| Jimmy y Timmy | Cinco teclas y rondas de 2, 3, 4, 5 y 6 notas. Las demostraciones bloquean la entrada. | Cinco rondas / tres errores. Un error repite la misma ronda. |
| Wendy y Bebe | Tablero ilustrado del pueblo, ruta continua de 28 s y siete alcantarillas. Algunas empiezan abiertas y las demás se abren con unos cuatro segundos de anticipación. | Llegar a casa / pasar por una alcantarilla abierta. |

El tablero de alcantarillas interpreta la cuadrícula de calles, casas, escuela y vegetación de South Park. No pretende ser una copia geográfica de cada edificio: es un tablero narrativo independiente. La proyección y los marcadores del minimapa 3D no se modificaron.

## Dirección artística

Renderizado nativo 2D de raylib, con ilustraciones vectoriales dibujadas en código y los recortes originales del proyecto. Sin HTML, frameworks ni nuevas dependencias.

- Paleta común crema, papel claro, tinta marrón, rojo cálido, azul grisáceo, verde apagado y amarillo suave.
- Tipografías existentes Gloria Hallelujah y Titillium Web; atlas con acentos y puntuación utilizados en ambos idiomas.
- Composición de libro de acertijos: instrucciones y personajes en un lateral; escena interactiva dominante; encabezado y controles inferiores compartidos.
- Cableado y calefactores en una caja de conexiones; luces dentro del OVNI; palancas en una consola escolar; teclas en el piano suspendido; tapas dentro de las calles ilustradas.
- Kenny visible en los cinco peligros, con temblor, elevación, caída del piano, impacto del autobús y descenso en las alcantarillas. Retratos originales sin deformación y parejas equilibradas.
- Sombras de contacto discretas, relieve suave, hover y presión en botones, luces pulsantes, estados de error y pequeños destellos de rescate.
- Layout compacto con instrucciones abreviadas y texto aumentado, en lugar de encoger todas las frases largas. Se comprobaron capturas a 640×480, 800×600, 1280×800 y 1920×1080.

## Arquitectura y archivos

- `src/minigames/southpark/puzzles.rs`: reglas puras, conectividad, secuencias, intentos, relojes y resultados. No tiene recursos gráficos, audio, listeners ni temporizadores del sistema operativo.
- `src/scenes/puzzle_ui.rs`: layout y áreas de clic compartidos; fuentes, paleta, ilustraciones, retratos prestados, Kenny, instrucciones bilingües, botones, pausa, resultados y preferencia musical.
- `src/minigames/southpark/session.rs`: un único `Option<PuzzleRun>`, apertura por encuentro, suspensión del mundo, consumo único de resultados, vidas, misión completada y retorno. La reacción de Stan/Kyle usa los mismos estados, reloj y pista de voz existentes. Las herramientas de prueba de peligros 3D anteriores se conservan.
- `src/scenes/southpark_dev.rs`: integración del render y los controles, entrega de música una vez por apertura/reinicio y opciones de smoke test. Omite el render 3D mientras el puzzle lo cubre, conservando los recursos cargados. Suspende el ciclo de Ike durante el modal para no eliminarlo ni recolocarlo.
- `src/scenes/encounter_ui.rs` y `session_ui.rs`: acceso de solo lectura a los retratos ya cargados; no se vuelven a decodificar por puzzle.
- `src/audio.rs`: mute del fondo musical, separado de efectos y voces, y reinicio explícito de la pista existente `assets/audio/music/South Park/save_kenny.mp3`. No se duplicaron ni editaron MP3. La nueva ruta no emite además el antiguo aviso Danger al abrir el puzzle.
- `src/scenes/puzzle_smoke.rs`: recorrido de victorias y derrotas reales con capturas y reacción de Stan/Kyle.
- `src/minigames/southpark/mod.rs` y `src/scenes/mod.rs`: registro de módulos.
- `tests/southpark_puzzles.rs`: soluciones, conectividad, unicidad lógica, límites, demostraciones, reinicios y audio.
- `tests/save_kenny_encounters.rs`: encuentro → puzzle → éxito/fallo → retorno, conservación del mundo, cinco rescates en distinto orden y derrota por tres vidas consumidas.

## Controles y ciclo de vida

1. E abre y avanza conversaciones; clic también avanza. L conserva el selector English/Español y su persistencia de fase 1.
2. Al terminar el guion, se abre la escena con un fade de 320 ms. Los relojes no corren durante la transición ni mientras se leen las instrucciones.
3. El botón de inicio o Enter comienza el puzzle. Solo entonces se aceptan acciones del tablero.
4. Pausa/Esc detiene animaciones y relojes. El botón de volver durante el juego abre esa pausa; confirmar volver deja la misión pendiente y no cuesta una vida.
5. Reiniciar reconstruye únicamente el estado del puzzle. No reinicia la ciudad ni duplica recursos. Tras un fallo permite reintentar si quedan vidas.
6. El resultado se registra una sola vez. La derrota reproduce la reacción original antes de descontar una vida; no vuelve a cobrarla mientras se permanece en la pantalla de fracaso.
7. Continuar devuelve al pueblo; el quinto rescate entrega `Victory` al host existente. Agotar las tres vidas entrega `Defeat`. Tokens y recompensas siguen usando el mismo `AttemptReporter` y las reglas existentes.

El mute se guarda en `saves/save-kenny-music.txt`; el idioma continúa en `saves/save-kenny-language.txt`. El mute musical no silencia las voces de diálogo, Stan/Kyle ni los efectos de resultado. Pausar/reanudar no vuelve a iniciar el canto. Abrir o reiniciar un puzzle sí reinicia la misma pista, salvo que esté silenciada.

## Verificaciones ejecutadas

Compilación: `cargo build --bins`.

Pruebas de reglas e integración:

```powershell
cargo test --test southpark_puzzles --test save_kenny_encounters --test save_kenny_session
cargo test --lib illustrated_controls_and_hit_regions_agree_at_all_supported_sizes
cargo test --test southpark_ike --test southpark_fps --test southpark_world --test southpark_threats --test southpark_phase6b --test progress_rewards --test attempt_rewards
```

La prueba de layout comprueba las posiciones de clic a las cuatro resoluciones indicadas y a 800×1000. La revisión visual mediante capturas se hizo en las cuatro resoluciones de escritorio; no se afirma haber revisado visualmente 800×1000.

Pruebas gráficas con OpenGL y eventos nativos:

```powershell
.\target\debug\southpark.exe --smoke-puzzles --mute
.\target\debug\southpark.exe --smoke-puzzle-results --mute
.\target\debug\southpark.exe --smoke-encounters --mute
.\target\debug\southpark.exe --smoke-ike --mute
```

- `--smoke-puzzles`: cinco conversaciones finales abren los cinco puzzles, botón de inicio, clics de tablero, cuatro resoluciones, ambos idiomas, música una vez por apertura, mute mediante clic y persistencia. Comprueba que WASD, M y movimiento de ratón no cambian la cámara/mundo, y que Ike conserva su posición. Sus preferencias de audio se escriben en `target/`, sin tocar las del usuario.
- `--smoke-puzzle-results`: resuelve los cinco puzzles mediante sus acciones reales y provoca sus cinco derrotas sin fabricar resultados. Captura rescate, peligro, Stan, Kyle y página de resultado; vuelve al mundo entre casos.
- `--smoke-encounters`: ocho retratos, diálogos bilingües, E sin saltar la primera frase, clic, bloqueo de movimiento y cierre. Conserva los tres audios de personajes.
- `--smoke-ike`: interacción, video, movimiento durante su reproducción, repetición y limpieza del flujo legado.

Registros: `target/puzzles-smoke.log`, `target/puzzle-results-smoke.log`, `target/encounters-phase2-smoke.log`, `target/ike-phase2-smoke.log`.

Capturas revisables: `target/southpark-puzzles-smoke/`. `puzzle-0..4-0..3.png` usa el orden calefactores, OVNI, autobús, piano, alcantarillas; el último número identifica la resolución. `outcome-0..4-*` son victorias y `outcome-5..9-*` derrotas.

## Límites y pulido opcional

Las pruebas de audio se ejecutaron silenciadas: verifican carga, canales, disparos, pausa y mute, pero no sustituyen la escucha humana. Tampoco se afirma una partida completa con teclado/ratón físicos ni una medición de 60 FPS en todos los dispositivos.

No quedan mecánicas de esta fase sustituidas por los antiguos clics 3D. Como mejoras opcionales: más tableros de cables validados, variantes de pistas, nuevas secuencias y tonos originales para acompañar las teclas. La memoria actual es visual y no añade melodías con copyright.

Sin cambios de rama, commits, merges ni push. Sin cambios en los recursos originales, otros minijuegos ni reglas de recompensas.
