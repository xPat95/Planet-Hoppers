# SAVE KENNY! — fase 6A

## Ejecutar y jugar

```powershell
cargo run
```

La exploración comienza con Kenny en uno de nueve puntos seguros del pueblo. Camina con el controlador y grafo existentes. Su marcador permanece oculto durante el paseo; encontrarlo, acercarse, apuntar y pulsar E permite iniciar el desafío. F3 conserva la ayuda opcional de distancia.

La partida mantiene tres vidas, cinco salvadas, las cinco amenazas y sus reglas de fase 5. **No hay botón Siguiente amenaza en el flujo normal.** Después de aceptar comienza un intervalo de exploración de 6 s; aparece automáticamente una alerta y se revela la ubicación de Kenny. La siguiente amenaza se prepara después de cada éxito; el mismo tipo pendiente se reintenta si falla.

| Control | Acción |
| --- | --- |
| WASD / mouse / Shift | Caminar / mirar / correr |
| E | Hablar o interactuar con el objetivo visible |
| M | Abrir/cerrar mapa ampliado |
| Arrastrar con botón izquierdo | Desplazar el mapa |
| Rueda | Zoom del mapa, entre 1× y 6× |
| R | Recentrar el mapa sobre el jugador, dentro de sus límites |
| Escape | Cerrar mapa; fuera de él, liberar/recuperar cursor FPS |
| P | Pausa y opción de abandonar el desafío |
| F1 / F2 / F3 / F12 | Ayuda / colisiones / distancia de Kenny / captura |
| Ctrl+Q | Salir |

No requiere numpad ni Blender para jugar. Escape fuera del mapa conserva su comportamiento anterior: libera el cursor, pero no pausa el peligro. P o M sí detienen la simulación.

## Flujo automático y búsqueda

El mismo `SaveKennySession` incorpora `Interlude`, `Relocating`, `Searching` y `MapOpen`; no se creó otro administrador ni se reimplementaron amenazas, FPS o raycasting.

```text
Hablar / comenzar -> Interlude -> Relocating -> Searching
Searching -> Preparing al acercarse con línea de visión -> Active
Éxito -> Success -> Interlude -> siguiente amenaza automáticamente
Muerte -> animación y voces -> descontar una vida -> nuevo Searching del tipo pendiente
5 salvadas -> Victoria; 3 muertes -> Derrota
```

La búsqueda permite desplazarse por toda la ciudad **sin tiempo mortal ni pérdida de vidas**. Kenny y los objetos ya están colocados; autobús, OVNI, piano y perseguidor permanecen detenidos. Solo al estar a 18 m o menos de Kenny y existir una línea de visión despejada empieza la preparación de 3 s. Después comienza la amenaza real. El jugador no se teletransporta al acceso.

La alerta muestra **¡KENNY ESTÁ EN PELIGRO!**, sector y distancia calculada desde las posiciones reales. Los mecanismos conservan sus alcances y oclusión del raycasting existente.

Una transición opaca breve de 0.4 s cubre la colocación y limpieza de Kenny/objetos; no es un intro cinematográfico. Evita mostrar saltos entre zonas. En el paseo no cambia de ubicación continuamente: se elige una aparición al cargar y al volver a una nueva exploración desde los menús. El movimiento posterior sigue sus waypoints normales.

Parámetros añadidos a `config/southpark-session.json`:

```json
{"world_interval_seconds":6.0,"arrival_distance":18.0,"relocation_seconds":0.4}
```

Se preservan los parámetros de preparación del director y los tiempos de audio de fase 5. Reiniciar para aplicar ajustes.

## Ubicaciones verificadas

`WorldLocations` obtiene coordenadas de calles comprobadas en el manifiesto, usando estaciones longitudinales predefinidas. Cada zona declara identificador, calle, centro de referencia, traslación, acceso y tipos compatibles. Las posiciones de Kenny, rutas y mecanismos provienen de la plantilla existente trasladada por ese desplazamiento, sin cambiar escalas, materiales ni orientaciones.

| Zona | Centro de referencia (X, Y, Z), metros |
| --- | --- |
| North Avenue oeste | (-145, 0, -42.9) |
| Central Avenue oeste | (-170, 0, 2.2) |
| Residential Avenue oeste | (-150, 0, 44) |
| South Neighborhood este | (80, 0, 66.55) |
| Infill Main | (35, 0, -23.1) |

**Cinco zonas válidas por amenaza:** autobús 5, OVNI 5, alcantarilla 5, piano 5 y muñeco 5. Son 25 combinaciones verificadas. El centro es la referencia local del evento; los puntos exactos de Kenny y cada objeto se calculan desde la plantilla del tipo elegido.

Se comprueban el recorrido de Kenny, el barrido completo del autobús, columna de aire/elevación del OVNI, superficies de tapa/agujero, volumen de caída/soporte del piano, ruta con el volumen grande del muñeco y accesos visibles a cada mecanismo. Los accesos tienen un recorrido transitable desde la red del pueblo, validado con el volumen humano. Se reutiliza `RoadNetwork` y su barrido; las búsquedas de ruta se hacen al preparar/validar, no continuamente por frame.

Kenny tiene nueve puntos de exploración: los anteriores y estaciones adicionales al este de North/Central/Residential y al oeste de South Neighborhood. Todas están sobre calles, conectadas al grafo y fuera de sólidos. La selección comprueba también la ocupación actual del jugador. Se evita repetir la última zona de evento cuando hay alternativas, incluso al reintentar una amenaza fallida.

La implementación usa traslaciones sobre avenidas horizontales. No intenta adaptar estas plantillas a curvas o posiciones arbitrarias. El catálogo exige al menos dos zonas comprobadas por tipo y admite hasta cinco; cambios incompatibles en la ciudad generan un error de validación, no apariciones dentro de edificios.

Auditoría sin ventana gráfica:

```powershell
cargo run --locked --bin southpark-locations
```

Produce `target/southpark-locations.json`. La [auditoría entregada](../references/southpark/phase6a/locations-validation.json) contiene las coordenadas completas.

## Imagen derivada y coordenadas

`tools/southpark/generate_map.py` genera `assets/textures/minigames/South Park/southpark_map.png` y su JSON a partir de `southpark_town.scene.json`. Dibuja las 15 polilíneas de calles y las 395 huellas de edificios, árboles, montañas y decoración. No abre ni guarda Blender ni modifica imágenes originales. Las huellas son simplificaciones cenitales de sus cajas geométricas, no techos detallados.

```powershell
python tools/southpark/generate_map.py
python tools/southpark/generate_map.py --check
```

La herramienta usa Pillow, disponible en el entorno de preparación; no agrega dependencias a Rust. `--check` compara SHA-256 del manifiesto y PNG. Regenerar si cambian calles o geometría del manifiesto. El JSON registra ejes, límites, resolución y procedencia.

La textura es de **1024×512**, aproximadamente 2 MiB de GPU en RGBA, y se carga una sola vez. No se regenera por frame. Los límites reales son X=[-350,370], Z=[-175,160]. La conversión es:

```text
Blender (x,y,z) -> mundo (x,z,-y)
u = (mundo.x + 350) / 720
v = (mundo.z + 175) / 335
```

X crece hacia la derecha; Z hacia abajo; el norte es -Z. Por ello una mirada +X apunta la flecha a la derecha y +Z hacia abajo. La altura de cámara o abducción no cambia la posición horizontal del marcador. El cargador comprueba que dimensiones y límites del mapa coincidan con el escenario.

## Minimapa y mapa ampliado

El minimapa circular sigue al jugador en la esquina inferior izquierda, con norte fijo y un radio visible de 65 m. Un shader recorta la textura al círculo. Muestra calles/bloques reales, flecha azul según yaw, Kenny naranja con K y peligro rojo con !. Kenny solo se revela durante búsqueda/preparación/amenaza; el peligro desaparece al terminar. Los objetivos lejanos se proyectan al borde del círculo. Los marcadores usan la posición actual de Kenny y la posición real del peligro: vehículo, OVNI, agujero, impacto o perseguidor.

M abre el mapa completo, inicialmente ajustado a toda la ciudad. El zoom se limita a 1–6, el arrastre se limita a los márgenes útiles y R centra en el jugador hasta donde permitan los bordes. Se usa la resolución actual de ventana. No hace falta cargar otros modelos ni recalcular la geometría del mapa.

`MapOpen` utiliza la misma suspensión de estado que la pausa: conserva el estado y su tiempo anterior. No se actualizan FPS, Kenny, director, animaciones, búsqueda ni intervalos; tampoco hay interacción E. La música puede continuar porque M está prohibida durante las voces. Cerrar con M/Escape restaura el estado, captura el cursor y descarta el delta inicial del mouse, sin reiniciar el evento ni emitir otra alerta.

Prioridad: diálogos de Kenny/Stan/Kyle, victoria, derrota, pausa y transición opaca bloquean M. El mapa consume mouse/teclado mientras está abierto. En exploración libre también congela el recorrido de Kenny. Los cuadros de diálogo no quedan tapados por el minimapa.

## Desarrollo y verificación

Se conserva `cargo run -- --dev-threats` y `--threat bus|ufo|sewer|piano|snowman`, con la plantilla original de desarrollo y F5/F6/F7/F10/F11, F8 y F9. No consumen vidas ni suman salvadas. La modalidad histórica de fase 5 y su botón de avance quedan restringidos al flujo de desarrollo/pruebas; el inicio normal habilita automáticamente las ubicaciones dinámicas.

```powershell
cargo fmt --all -- --check
cargo check --locked
cargo build --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --locked -- --smoke-world
```

Pasaron **82 pruebas** (71 anteriores y 11 nuevas). Las nuevas cubren nueve apariciones seguras, selección sin repetición inmediata, éxito y fracaso de las 25 combinaciones, búsqueda prolongada sin peligro, victoria automática sin llamadas de avance, pausa/reanudación de las cinco amenazas —incluida la caída del piano—, prioridad de menús y proyección/zoom/desplazamiento limitados.

La prueba gráfica `--smoke-world` usa OpenGL, eventos de entrada de raylib y el raycasting existente. Completa cinco salvadas automáticas en zonas variables, una derrota con tres reintentos del mismo tipo, retorno/reinicio y pausa. Abre/cierra el mapa en cada amenaza, pulsa W mientras está abierto y compara posiciones, yaw/pitch, temporizadores, progreso, balanceo, vehículo, OVNI, piano y perseguidor: permanecen congelados. Prueba M/Escape, rueda, arrastre, R y recuperación del cursor. Guarda capturas e informe aquí. La automatización posiciona y orienta al jugador; **no reemplaza una partida manual ni una revisión auditiva**.

También pasaron los modos `--smoke`, `--smoke-kenny`, `--smoke-threats` y `--smoke-session`: se conservaron FPS, colisiones, controles, retratos, audio, cinco amenazas y reglas de fase 5. Esos informes quedan en `target/southpark-*-smoke/` para preservar capturas históricas. Los PNG, MP3, Blender, modelos del planeta/ciudad/naves y amenazas originales se compararon por SHA-256 contra 87 archivos iniciales: sin cambios. Cargo.toml y Cargo.lock permanecen intactos.

Capturas: [minimapa/HUD](../references/southpark/phase6a/hud.png), [búsqueda](../references/southpark/phase6a/search.png), [mapa completo](../references/southpark/phase6a/map.png), [mapa con zoom](../references/southpark/phase6a/map-zoom.png). Se incluyen también diálogo, victoria, derrota y pausa. Informes: [ejecución dinámica](../references/southpark/phase6a/runtime-validation.json), [regresiones](../references/southpark/phase6a/regression-validation.json).

Quedan por revisar manualmente comodidad del recorrido, dificultad, lectura del mapa en otras resoluciones y teclado/mouse físicos. La sincronización auditiva de Stan/Kyle conserva el valor inicial de fase 5 y requiere escucha. El mapa representa cajas simplificadas y las zonas se limitan a las avenidas verificadas; una modificación del pueblo requiere regenerar y auditar.

## Archivos y fase 6B

Creados: `src/minigames/southpark/locations.rs`, `src/minigames/common/world_map.rs`, `src/scenes/world_map.rs`, `src/graphics/shaders/minimap.fs`, `src/bin/southpark-locations.rs`, `tests/southpark_world.rs`, `tools/southpark/generate_map.py`, PNG/JSON derivados del mapa, este documento y referencias de fase 6A.

Modificados: `README.md`, `config/southpark-session.json`, `src/minigames/southpark/{mod,session,threats}.rs`, `src/minigames/common/mod.rs`, `src/graphics/threats.rs`, `src/scenes/{mod,session_ui,session_smoke,southpark_dev}.rs`. No se agregaron dependencias ni se reescribieron los controladores generales.

Fase 6B conserva pendientes el intro, reproducción de `intro.mp3` e integración con sistema solar/nave. No hay nuevos personajes, amenazas, multijugador ni guardado. No hubo commit, push, cambio de rama, merge ni rebase.
