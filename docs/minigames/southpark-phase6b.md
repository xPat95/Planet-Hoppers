# South Park — fase 6B

## Ejecutar y probar

Desde la raíz del repositorio:

```powershell
cargo run
```

La primera entrada muestra la ciudad real en 3D con `SOUTH PARK` y `intro.mp3`.
Espacio o Enter saltan la presentación. Kenny aparece visible a unos **7,2 m** del
jugador inicial. Caminá hacia él, apuntá, presioná E y aceptá el desafío.

Controles conservados: WASD, mouse, Shift para correr, E para interactuar,
Escape para liberar/recuperar el cursor y P para pausar/abandonar una partida.
M abre/cierra el mapa; rueda para zoom, arrastre izquierdo para desplazar,
R para recentrar, Escape para cerrar. **Retroceso** sale de South Park durante
exploración libre. En el ejecutable independiente esto cierra la aplicación;
no muestra un sistema solar ficticio. F12 guarda `target/southpark.png`.

```powershell
cargo run -- --dev-threats
cargo run -- --threat snowman
cargo run -- --smoke-entry --mute
cargo run -- --smoke-world --mute
cargo test --locked
```

El modo de desarrollo mantiene F5/F6/F7/F10/F11 y F8/F9. Omite la introducción
para facilitar las pruebas y conserva las ubicaciones canónicas individuales.
No modifica vidas ni salvadas de una partida normal.

## Inspección y arquitectura

Se conserva Rust 2021 y raylib-rs **5.5.1**, sin nuevas dependencias.
`src/bin/southpark.rs` llama al anfitrión `scenes/southpark_dev.rs`.
El FPS, volumen barrido AABB y raycasting siguen en `minigames/common`.
`SaveKennySession` sigue coordinando las cinco amenazas de `ThreatDirector`;
Kenny, UI, audio y mapas conservan sus módulos separados.

El manifiesto mantiene el suelo Y=0, metros, X horizontal y Z horizontal;
la conversión desde Blender es `(x,z,-y)`. La ciudad mide 720×335 m y conserva
395 sólidos, 15 calles y la red existente. No se tocaron modelos, PNG ni MP3.

`src/solarSystem/` y `src/core/` solo contienen `.gitkeep`: no hay selección de
planeta, pilotaje ni gestor de escenas ejecutable en esta rama. No se creó un
sistema solar ni se modificó otro planeta o nave.

## Presentación y memoria de visita

`scenes/southpark_entry.rs` contiene la secuencia de entrada y su configuración
está en `config/southpark-entry.json`:

- MP3 existente: **27,376326 s**, medido por decodificación nativa de raylib
  y contrastado leyendo las cabeceras de sus 1048 frames MP3.
- Presentación completa: **27,376326 s**, obtenidos del audio cargado, sin un
  límite que lo recorte. La cámara usa el reloj del stream y la secuencia termina
  al finalizar su reproducción, sin espera adicional. Si falta audio, usa
  `fallback_intro_seconds` (27,376326 s).
- Visitas posteriores: **0,7 s**, sin música de introducción.

Las cuatro posiciones de cámara son `(-180,110,95)`, `(-95,75,-42.9)`,
`(spawn.x,40,spawn.z)` y el ojo FPS exacto. Hay objetivos de mirada propios
para cada toma, interpolación smoothstep, barras cinematográficas y fundido final.
La trayectoria se valida con 1001 muestras contra los sólidos con margen de
15 cm. La toma general permite reconocer calles, casas, montañas y el cartel;
no se añadió una toma cercana específica del cartel.

Durante la presentación no avanzan Kenny, jugador, interacciones ni partida.
La escena sigue renderizándose y el stream de música sigue actualizándose.
Al terminar o saltar, se detiene el stream de intro, se usa la cámara FPS,
se captura el cursor y comienza/reanuda la música de exploración. El MP3 termina
al finalizar naturalmente, o al saltar explícitamente la presentación. El archivo
no se modifica. El reloj del stream evita que los FPS adelanten la cámara o
recorten las últimas notas; la prueba cubre también una posición de audio en el
final nominal que todavía no informa haber terminado.

`SouthParkVisits::intro_seen` pertenece al anfitrión, no a SAVE KENNY!.
Se marca al completar o saltar la entrada; reiniciar una partida no lo cambia.
No existe persistencia en disco. La prueba gráfica fuerza primeras visitas
adicionales únicamente para comprobar ambos botones de salto.

## Kenny al entrar y al regresar a exploración

`place_kenny_near_player` busca nodos transitables a 4–14 m delante del jugador,
con línea de visión y acceso por calles, comprobando el volumen de Kenny y la
separación del jugador. Como respaldo busca nodos accesibles a 4–24 m.
No crea coordenadas aleatorias sin validar. Reutiliza `relocate_kenny` y reanuda
su controlador normal. La primera colocación ocurre antes del intro; los
retornos tras ganar, perder o abandonar usan la transición de recolocación.
Después no se recoloca continuamente mientras camina.

Las cinco zonas por amenaza y los nueve puntos del catálogo de fase 6A siguen
existiendo. Durante la partida la selección evita repetir la zona anterior y
Kenny puede aparecer lejos del jugador; no se sustituyó por un desafío local.

## Activación híbrida y balance

Flujo normal: intervalo de exploración → recolocación oculta → búsqueda con
plazo → preparación → amenaza activa → resultado. La preparación comienza
al entrar en **32 m con visión directa**, o al expirar el plazo, lo que ocurra
primero. Alejarse después no cancela ni reinicia la preparación. El peligro
real empieza al terminar los **2 s** de advertencia; ningún evento espera
indefinidamente la proximidad del jugador. Los controles continúan disponibles.

**Ajuste posterior: máximo de 20 s desde la aparición jugable de la amenaza.**
El reloj inicia en cero cuando los objetos y rutas están preparados y el FPS
recupera el movimiento. Búsqueda y preparación comparten ese reloj; el intervalo,
la transición y el aviso de éxito anterior no lo consumen. Al generar cada
reintento se reinicia en cero. M y P conservan su pausa del mundo.
Ver [detalle y verificaciones del límite total](southpark-activation-deadline.md).

Parámetros en `config/southpark-session.json`:

| Parámetro | Valor | Uso |
| --- | --- | --- |
| `zone_detection_distance` | 90 m | Indica que ya se detectó la zona en el aviso |
| `arrival_distance` | 32 m | Adelanta la preparación con visión directa |
| `activation_max_seconds` | 20 s | Máximo desde la amenaza generada y los controles disponibles |
| `search_min_seconds` / `search_max_seconds` | 2 / 20 s | Límites orientativos, sujetos al presupuesto restante |
| `search_route_factor` | 1,05 | Margen sobre el tiempo estimado corriendo |
| `search_orientation_seconds` | 3 s | Tiempo adicional para orientarse |
| `world_interval_seconds` | 6 s | Intervalo entre eventos |
| `relocation_seconds` | 0,4 s | Transición que oculta la colocación |

Se calcula una vez:

```text
estimación = clamp(longitud_ruta / velocidad_carrera × 1,05 + 3, 2, 20)
búsqueda = min(estimación, max(0, 20 - tiempo_acumulado - preparación))
```

Se reutiliza `event_route` sobre la red de calles. Se prueban el acceso original
y sus dos laterales a ±3 m, eligiendo la ruta más corta de las encontradas.
Se incluyen los obstáculos temporales ya colocados: un autobús estacionado
puede obligar a un rodeo importante. Se ignora el cuerpo de Kenny para estimar
el acceso; el controlador del jugador sigue colisionando con él al caminar.
Si no se encuentra ruta, el respaldo documentado es distancia recta×1,8.
La búsqueda no recalcula ni extiende su plazo al moverse el jugador.

El cálculo por distancia nunca amplía el máximo total. Con los valores actuales,
la búsqueda dispone de hasta 18 s desde la generación con movimiento disponible;
los 2 s de preparación están incluidos en los 20 s. Intervalo, generación y
transición ya no descuentan tiempo de ese reloj. Los recorridos
históricos desde el spawn inicial alcanzaban 89–531 m: el nuevo límite exige
estar bien ubicado y no garantiza salvar a Kenny desde cualquier extremo de
la ciudad. No se cambiaron la velocidad del jugador ni las de las amenazas.

Balance en `config/southpark-threats.json`:

| Amenaza | Ajustes actuales | Fracaso sin intervención |
| --- | --- | --- |
| Autobús | 10 s; recorrido 52,85 m, ≈5,29 m/s | Contacto con Kenny, cerca del final del recorrido |
| OVNI | 12 s; ascenso 4,5 m, ≈0,375 m/s | Abducción al completar el ascenso |
| Alcantarilla | 12 s para caminar 14 m | Caída al llegar sin tapa |
| Piano | 11 s y caída de 0,65 s desde 6,5 m | Aplastamiento sin bloqueo |
| Muñeco | 2,35 m/s; Kenny 1,25 m/s; separación 12 m | Contacto del perseguidor |

Cada calefactor reduce la velocidad del muñeco en 0,07 m/s; los tres lo
detienen y derriten durante 2 s, como antes. El alcance de interacción es
4,5 m; las luces del OVNI mantienen 14 m. Las mecánicas, recursos, restauración,
animaciones de fracaso y resultados existentes se reutilizan.

Se mantienen tres vidas, cinco salvadas distintas, reintento de la amenaza
fallida, orden aleatorio, avance automático y consumo único de cada resultado.
El diálogo de Stan/Kyle conserva el único MP3, cambio de retrato a 2,18 s y
duración real de voz de 3,343674 s. No se añadió otro sistema de raycasting.

## Mapas y audio

El minimapa sigue usando la textura derivada de las calles y edificios reales.
No se modifica ni se regenera por frame. Sus coordenadas continúan siendo
X hacia la derecha, Z hacia abajo y norte = −Z. Kenny y el peligro usan sus
posiciones reales y los objetivos lejanos conservan el indicador del borde.

`MapOpen` conserva el estado y tiempo anterior. Pausa también el nuevo plazo
de búsqueda, además de FPS, Kenny, temporizadores y actores de las cinco
amenazas. Cerrar el mapa restaura esos valores sin reinicio. Los diálogos,
victoria, derrota e intro tienen prioridad sobre M.

`AudioController` incorpora `MusicMode::Intro` y un stream no repetitivo.
Su política idempotente evita reiniciarlo por frame. Exploración, desafío,
efectos, pasos y voz conservan sus canales y volúmenes configurables. La salida
silencia música, efectos y voz; los recursos se liberan por sus propietarios.
Archivos o dispositivo ausentes siguen produciendo advertencias sin bloquear
la escena. No se reemplazó ni editó ningún audio.

## Contrato para el futuro anfitrión solar

```rust,ignore
let mut visits = SouthParkVisits::default(); // conservar durante la sesión global
let exit = southpark_dev::run_visit(&mut visits)?;
// ReturnToHost: restaurar la escena que hizo la llamada.
// WindowClosed: cerrar la aplicación según la política del anfitrión.
```

`run_visit` es un adaptador independiente que actualmente posee su ventana
y sus recursos raylib. No se debe abrir una segunda ventana mientras el futuro
anfitrión mantenga otra activa. Cuando exista un gestor con ventana compartida,
el equipo deberá conectar su carga/dibujo/descarga con `SouthParkScene`,
`SouthParkEntry`, `SaveKennySession` y los recursos gráficos/audio existentes,
conservando `SouthParkVisits` fuera de la escena.

Pendiente para la integración definitiva: acción «Explorar planeta», cesión de
cámara/cursor/audio, llamada de retorno, conservación de la nave anterior y
notificación de `Victory` al progreso global. Tokens/desbloqueos deben seguir
perteneciendo a gameplay; esta fase no inventa sus reglas ni entrega recompensas.

## Verificación y evidencia

- `cargo check --locked`, `cargo build --locked`, `cargo fmt --all -- --check`:
  correctos.
- `cargo test --locked`: **90 pruebas correctas**, incluyendo las ocho nuevas
  de fase 6B y las regresiones FPS, Kenny, partida, amenazas y mapas.
- `cargo clippy --all-targets --locked -- -D warnings`: correcto.
- `python tools/southpark/generate_map.py --check`: correcto.
- Inactividad: **25/25** (cinco amenazas × cinco zonas). Vence búsqueda,
  activa peligro, ejecuta fracaso/animación/diálogo, resta exactamente una vida,
  emite una sola señal de voz y mantiene cero salvadas.
- Rescate: **25/25**, desde accesos cercanos a las zonas. Movimiento con el controlador FPS
  a 8 m/s y colisiones, aproximaciones transitables, raycasting compartido con
  refinamiento y oclusión, reacción de 0,5 s entre acciones. Sin teletransporte
  durante los recorridos. Una salvada, tres vidas conservadas en cada caso.
- Se comprueba que M preserve exactamente el plazo durante 200 s simulados y
  que salir de la zona después de comenzar preparación no reinicie el evento.
- `--smoke-world --mute`: OpenGL real, E y menús por eventos nativos; partida
  ganada, derrota de tres vidas, reintento, exploración y cinco pausas/reanudaciones
  de mapa con zoom y arrastre. Esta prueba gráfica automatiza colocación y
  puntería; las pruebas de recorridos anteriores verifican el desplazamiento.
- `--smoke-entry --mute`: intro completo con delta real, saltos Espacio/Enter,
  transición de visita posterior, captura del cursor y bloqueo de W/M/E.
- Pruebas gráficas anteriores `--smoke`, `--smoke-kenny`, `--smoke-threats` y
  `--smoke-session`: correctas.
- Los **145 recursos originales versionados en assets** conservan sus SHA-256.

Evidencia en [phase6b](../references/southpark/phase6b/): capturas del intro,
aviso, HUD, mapas, diálogos y finales; `intro-validation.json`,
`runtime-validation.json`, `idle-validation.json`, `rescue-validation.json`
y `regression-validation.json`.

Esos archivos conservan la evidencia histórica de la entrega 6B. Las pruebas del
ajuste posterior a 20 s están en
[activation-deadline](../references/southpark/activation-deadline/validation.json).

Las pruebas gráficas se ejecutaron silenciadas. Se verificaron carga, streams,
transiciones y retratos, pero **no se escuchó la mezcla ni la sincronización**.
La comodidad de los controles físicos y la dificultad humana requieren prueba
manual: la puntería automatizada no reproduce dudas o errores de una persona.
Los recorridos de rescate actuales abarcan las 25 combinaciones desde accesos cercanos;
no garantizan llegar desde el spawn inicial o todas las posiciones posibles entre eventos ni todos los tamaños de
ventana. El mapa conserva sus huellas AABB simplificadas.

## Archivos

Creados:

- `config/southpark-entry.json`.
- `src/scenes/southpark_entry.rs`.
- `tests/southpark_phase6b.rs`.
- `docs/minigames/southpark-phase6b.md`.
- `docs/references/southpark/phase6b/`: capturas y reportes de esta fase.

Modificados:

- `config/southpark-session.json`: radios y búsqueda limitada.
- `config/southpark-threats.json`: balance individual.
- `src/audio.rs`: stream y política de intro.
- `src/minigames/southpark/session.rs`: aparición cercana y activación híbrida.
- `src/scenes/southpark_dev.rs`: entrada, salida, bloqueo de controles y pruebas.
- `src/scenes/mod.rs`: exportación del componente de entrada.
- `src/scenes/session_ui.rs`: plazo y detección de zona.
- `src/scenes/session_smoke.rs`: auditoría del nuevo recurso de audio.
- `tests/save_kenny_session.rs`: fallo de carga opcional del noveno audio.
- `tests/southpark_threats.rs`: esperas basadas en los tiempos configurados.
- `tests/southpark_world.rs`: regresión actualizada a búsqueda finita.
- `README.md`: ejecución y referencia de fase 6B.

Cargo y sus dependencias permanecen intactos. La corrección de tiempos se guarda
en un commit local solicitado por el usuario; no se realiza push, cambio de rama,
merge ni rebase.
