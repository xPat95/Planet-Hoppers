# Ike: Kick the Baby en exploración

Los originales se conservan en `assets/textures/minigames/South Park/IkeBroflovski.png`
y `assets/video/south park/Kick_the_baby.mp4`. El PNG realmente se llama
**IkeBroflovski**, no `IkeBrofloski`.

## Corrección de derivados tras editar el MP4

El juego reproduce `kick_the_baby.sfv` y `kick_the_baby.ogg`, no el MP4 en tiempo
real. Después del recorte manual, el MP4 actual es 1080×1080 (SAR/DAR 1:1), con
SHA-256 `1adecfeb7c2b2c2b2e6daf8ed042648059164f612bf1d18567f4a3d265384268`.
El JSON anterior aún identificaba el origen `7be15a0f…`, y el SFV antiguo seguía
siendo 640×360, con las barras laterales anchas incorporadas en sus propios JPEG.
Esa era la causa de que el juego siguiera mostrando exactamente la versión vieja.

Se regeneraron únicamente los tres derivados `kick_the_baby.{sfv,ogg,json}`:
ahora el SFV es 640×640 y el plano se adapta automáticamente a esa proporción.
No se añadió recorte, padding, letterboxing, fondo negro ni deformación. El
MP4 actual todavía contiene franjas negras laterales estrechas; permanecen como
parte de la imagen suministrada. Los hashes del MP4 y de todos los otros videos,
incluida la intro principal, permanecieron iguales durante la corrección.

`python tools/video/prepare_southpark_ike.py --check` ahora detecta derivados
desactualizados mediante el SHA-256 del origen, hashes de SFV/Ogg y cabecera del
SFV. Falló correctamente antes de regenerar y pasó después (`IKE_DERIVED_OK`).
Si se vuelve a editar el MP4, ejecutar primero el conversor sin `--check`.
La prueba Rust ya no fija 640×360: compara dimensiones con los metadatos.
Se aprobaron las diez pruebas de Ike/video y `--smoke-ike --mute`; la captura
`target/southpark-ike-smoke/02-video-world.png` muestra el nuevo plano cuadrado
en el pueblo. La prueba comprueba también movimiento, final, repetición y
limpieza al iniciar el desafío. La escucha de audio sigue siendo manual.

## Implementación

- `src/minigames/southpark/ike.rs`: estado independiente, selección aleatoria entre
  `WorldLocations.exploration_spawns`, comprobación de espacio para el sprite y
  acceso mediante `event_route`. Mantiene más de 8 m de separación inicial de Kenny
  y 4 m del jugador. No registra colisionadores y no bloquea la navegación.
  La selección ocurre una vez por exploración; no vuelve a sortear cada frame.
- `src/graphics/billboard.rs`: carga opcional con recorte de márgenes transparentes
  únicamente en RAM. Ike mide 0,96 m y conserva proporciones y transparencia.
  La carga habitual de Kenny mantiene su comportamiento.
- `src/scenes/southpark_dev.rs`: añade a Ike a una lista temporal de candidatos
  para `select_target_with`, con impacto refinado por alfa y oclusión por sólidos.
  No lo agrega a los objetivos de la escena ni a las amenazas. Muestra
  `[E] Hablar con Ike`; utiliza el flanco de pulsación de E existente.
- `src/scenes/southpark_video.rs`: `VideoTexture` comparte la decodificación y la
  actualización de textura con la intro. El lector SFV indexa el archivo y lee solo
  el JPEG seleccionado. Se mantiene una textura GPU durante el clip, sin cargar
  todos los fotogramas en RAM. La intro conserva layout y controles.
- `src/scenes/southpark_ike.rs`: posee la textura y un único stream Ogg no cíclico.
  El tiempo del audio selecciona el fotograma; sin dispositivo utiliza tiempo real.
  La textura se dibuja con `draw_billboard_pro` dentro de `begin_mode3D`, anclada a
  la posición de Ike, centrada a 0,48 m de altura, con un máximo de 1,65 m de ancho
  y 1,05 m de alto, preservando la proporción de los fotogramas preparados.
- `src/audio.rs`: aplica un multiplicador temporal de 0,2 a la música de exploración
  y restaura el volumen configurado al terminar. No inicia `intro.mp3` ni modifica
  la configuración. WASD, correr y mirar con el mouse siguen disponibles.

La primera imagen se decodifica y sube antes de sustituir el sprite: no se dibuja
una superficie negra de carga. La finalización o cualquier error destruye el
reproductor y restaura a Ike en la misma posición. Se registra `IKE_VIDEO_WARNING`
si falla la carga/actualización. Solo puede existir un reproductor de Ike.

Solo `SessionState::Exploring`, sin intro activa, permite dibujar/interactuar.
Comenzar el desafío cancela el clip y elimina la posición. Mapa o pausa cancelan
la reproducción, preservando la posición para volver a explorar. Regresar al
sistema solar, cerrar la ventana y retornar por error liberan los recursos mediante
RAII/Drop. Una nueva exploración vuelve a sortear la ubicación. La aparición se
habilita en el modo normal; los modos técnicos de amenazas conservan sus escenas.

## Recursos derivados

`tools/video/prepare_southpark_ike.py` reutiliza el formato y las herramientas de la
intro. Genera `kick_the_baby.sfv`, `kick_the_baby.ogg` y `kick_the_baby.json` junto
al MP4: 640×640, 24 FPS, 91 fotogramas, aproximadamente 3,792 segundos.
El JSON registra el SHA-256 del original. FFmpeg solo se necesita para regenerar:

```powershell
python tools/video/prepare_southpark_ike.py
```

Configurar `FFMPEG_PATH` o instalar `imageio-ffmpeg` si no está disponible.
Los tres derivados deben distribuirse con el juego; no se decodifica MP4 en tiempo
real. Sin dispositivo de audio el evento sigue funcionando sin sonido.

## Verificación automatizada y gráfica

```powershell
cargo check
cargo build
cargo test --test southpark_ike --test southpark_video --test southpark_world --test save_kenny_session --test southpark_threats --test kenny_phase3 --test southpark_phase6b
cargo run --bin southpark -- --smoke-ike --mute
cargo run --bin southpark -- --smoke-entry --mute
```

`tests/southpark_ike.rs` verifica variedad de ubicaciones, separación, suelo,
accesibilidad, estados, repetición, exclusión del desafío, transparencia, rango,
paredes y metadatos. `tests/southpark_video.rs` también comprueba el lector indexado
ante truncamiento y datos sobrantes.

`--smoke-ike` abre una ventana oculta, utiliza el mapa y recursos reales, aproxima
al jugador solo dentro de la prueba y pulsa E mediante automatización de raylib.
Comprueba reproducción, movimiento FPS, finalización, repetición y limpieza al
comenzar el desafío. Produce seis capturas en `target/southpark-ike-smoke/`.
El modo no cambia las coordenadas ni la cámara durante una partida normal.

Se inspeccionaron las capturas del sprite, video y desplazamiento. Las pruebas
con `--mute` comprueban el stream y reloj reales, pero no constituyen una escucha
del audio ni una medición de sincronía perceptual o de rendimiento en otros equipos.

Resultado de esta implementación: `cargo check` y `cargo build` correctos; 81
pruebas automatizadas aprobadas en las siete suites indicadas; `IKE_SMOKE_OK` en
la prueba gráfica. La prueba de intro verificó final natural, Espacio, Enter,
clic en Skip y visita posterior (`ENTRY_SMOKE_PASS`). Se revisaron `git diff`
y `git status` en la rama `feature/easter-egg`, sin commit ni push.

Archivos modificados: `src/audio.rs`, `src/graphics/billboard.rs`,
`src/minigames/southpark/mod.rs`, `src/scenes/mod.rs`,
`src/scenes/southpark_dev.rs`, `src/scenes/southpark_video.rs` y
`tests/southpark_video.rs`.
Archivos creados: `src/minigames/southpark/ike.rs`,
`src/scenes/southpark_ike.rs`, `tests/southpark_ike.rs`,
`tools/video/prepare_southpark_ike.py`, este documento y los tres derivados
`assets/video/south park/kick_the_baby.{sfv,ogg,json}`.
Los dos originales proporcionados permanecen sin seguimiento en Git, sin cambios.

## Prueba manual pendiente

1. Ejecutar `cargo run --bin southpark`, sin `--mute`. Ver o saltar la intro.
2. Recorrer el pueblo y buscar al pequeño Ike; la consola registra `IKE_SPAWN`
   con coordenadas para facilitar la comprobación. Verificar suelo y transparencia.
3. A menos de 4,5 m, apuntar al cuerpo: debe aparecer `[E] Hablar con Ike`.
   Comprobar que una casa/pared bloquea la interacción y que apuntar fuera no activa.
4. Pulsar E y mantenerla: un solo clip sustituye a Ike. Caminar lateralmente,
   mirar hacia otro lado y alejarse; el video permanece en el mismo lugar del pueblo.
5. Escuchar una sola pista del clip, comprobar sincronía y reducción/restauración
   de música. Esperar el final: Ike reaparece sin cambiar de ubicación. Repetir.
6. Durante otro clip, hablar con Kenny e iniciar SAVE KENNY!: desaparecen Ike y
   el video, se corta su audio y funcionan las amenazas, vidas, salvadas y HUD.
7. Volver a explorar; comprobar una nueva ubicación. Durante un clip, regresar al
   sistema solar y verificar que no sigue sonando. Repetir en una nueva visita.
8. Comprobar visualmente las cinco amenazas, diálogos, victoria/derrota, mapa y
   minimapa durante una partida completa. Las pruebas automáticas cubren sus reglas.

No se modificaron modelos, nave, sistema solar ni código de otros minijuegos.
