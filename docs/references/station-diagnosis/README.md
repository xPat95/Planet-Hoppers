# Diagnóstico de estación y corrección del N‑1

Trabajo sobre `main`, inicialmente limpio. Sin commit, push ni operaciones de integración de ramas. Hangar, plataforma, GLB/Blender originales, configuración, órbitas, menú y controles permanecen intactos.

## Causa comprobada

La barrera provenía de **`Station::push_out`**, que trataba como sólido todo el interior de una esfera de radio 860, salvo un corredor estrecho. No era un límite global del mundo.

La estación está en `(0,6200,0)`, con rotación `(0,-90,0)` y longitud ajustada 1400. La caja conservadora que se examinó anteriormente tenía semiejes aproximados `(700,317,332)`; el análisis directo de los vértices y jerarquía del GLB da `(700,298.214,311.101)`. Ninguna de esas formas justifica rellenar una esfera de radio 860: además de exceder la altura y el ancho del modelo, la esfera rellenaba los huecos entre sus brazos.

Se revisaron `Ship::update`, `resolve_collisions`, el sistema solar y la escena de vuelo. Las correcciones del controlador corresponden al Sol y a las formas de los planetas. No hay clamp de posición, radio máximo de vuelo ni pared mundial. `MAX_ZOOM_OUT`, los límites orbitales y los planos de recorte afectan a la cámara/render, no a la posición del jugador. Los cuerpos celestes y sus colisiones no se modificaron.

La regresión se reproduce con una nave inmóvil a 500 unidades por encima o al costado de la estación: el controlador conserva su posición, pero la antigua condición esférica la consideraba dentro de la estación. Ahora permanece libre en ambas posiciones, comprobado en tests y en el juego nativo.

## Colisión derivada del modelo

El nuevo recurso es `assets/models/environment/spaceStation/exports/citadel_space_station_near.collision.json`. Se genera leyendo el **GLB original**, sus índices de 32 bits, los 683.746 triángulos y las transformaciones de su jerarquía. El algoritmo marca solo las celdas que intersectan triángulos mediante pruebas de separación triángulo/caja. Une únicamente celdas contiguas de una misma fila; no rellena los huecos entre brazos ni convierte el anillo completo en un bloque.

Resultado: 253.812 celdas de cuatro unidades, comprimidas en 22.424 cajas y un JSON de aproximadamente 362 KiB. Contiene fuente, huella SHA‑256, longitud de referencia y datos de ajuste. El modelo original no se reexporta ni modifica.

En ejecución se aplican longitud, rotación y posición global de la estación. `align_collision` sincroniza el centro con la matriz de ajuste exacta del modelo renderizado. El offset medido en el juego es prácticamente cero. El barrido selecciona el primer contacto con las cajas ampliadas por el radio completo de la nave; evita atravesar superficies a alta velocidad. Si comienza dentro de una celda, sale de la unión de cajas superpuestas, evitando alternar correcciones entre celdas vecinas. El marco exterior existente tiene cuatro volúmenes conservadores adicionales en sus propios ejes.

La antigua esfera se conserva **solo como envolvente de separación del despegue**. Ya no bloquea el vuelo ni corrige la cámara. El despegue sigue validando nave, cámara y avance con las colisiones reales, además de su margen conservador existente.

La entrada se calcula frente al límite del recurso ajustado y al marco de la compuerta, contemplando el tamaño de la nave. La zona exterior mantiene un máximo de 40 unidades alrededor de esa ancla; se conserva también el acceso cercano por la abertura. E solo se habilita si el volumen de la nave está libre de colisión. Se mantienen cooldown, pulsación nueva y transiciones del hangar. Para entrar se sigue usando la compuerta frontal, no cualquier cara de la estación.

## Depuración: F3

Durante vuelo o vista general, **F3** alterna la visualización; está desactivada por defecto. F12 conserva la captura manual en `target/solarsystem.png`.

- Cian: hasta 512 cajas de las superficies ocupadas próximas a la nave.
- Rojo: volumen de la nave, usando sus dimensiones cargadas.
- Verde: ancla y rango exterior de interacción.
- Naranja: abertura y profundidad del marco exterior.
- Amarillo: antigua envolvente esférica, identificada explícitamente como referencia de despegue **sin colisión**.

El HUD muestra posición, radio, distancia al centro/ancla, habilitación de entrada, ausencia de límite mundial, última corrección de nave y motivo de corrección de cámara en el frame actual. Distingue estación, Sol y planetas. La última corrección permanece visible aunque ya se haya abandonado el contacto.

También puede iniciarse directamente con:

```powershell
cargo run --locked --bin solarsystem -- --debug-collisions
```

## Star Wars: orientación y presencia

El GLB preparado del N‑1 conserva frente `+Z`; el controlador y los modelos corregidos de vuelo esperan frente `−Z`. `ShipModel::flight_rotation_degrees()` añade 180° alrededor de Y **solo al cargador de vuelo**. La transformación de gameplay, yaw, pitch, bank, W y Espacio no se invierte. Los tests contrastan el frente visual con el vector forward en distintos yaw, pitch y bank, para las variantes Naboo y Mandaloriano.

La presentación del hangar sigue usando la rotación original del modelo, con su corrección de exhibición ya aprobada. No se cambia su ángulo, escala ni cámara.

El perfil trasero correcto resultaba más estrecho en pantalla. Para darle mayor presencia, el encuadre específico del N‑1 pasa de `0,90*longitud` a `0,60*longitud`: su seguimiento queda un tercio más próximo que en la versión anterior de esta tarea. La distancia horizontal actual es 8,28 unidades para longitud 5, frente a las 12,42 anteriores. Se conserva FOV 60°, suavizado y protección de cámara. No se aumenta la geometría ni la hitbox; la planificación del despegue utiliza la misma cámara real. Las otras naves mantienen su encuadre.

## Evidencia y pruebas

Capturas del juego real:

- [Espacio libre encima de la estación](debug-station-debug-above.png).
- [Espacio libre al costado](debug-station-debug-side.png).
- [Recuperación estable y superficies físicas](debug-station-debug-recovery.png).
- [Compuerta, rango y E habilitada](debug-station-contact.png).
- [N‑1 corregido en vuelo](starwars-launch-starwars.png), [versión anterior](../hangar-polish/launch-starwars.png).
- [N‑1 en el hangar conservado](hangar-starwars.png), [Smiling Friends en vuelo](smilingfriends-launch-smilingfriends.png).
- [Regreso con N‑1](return-starwars-dock-approach-0.png), [regreso con Smiling Friends](return-smilingfriends-dock-approach-0.png).
- [Nuevo despegue tras cambio de nave](return-smilingfriends-dock-launch-2.png).

Pasan `cargo check --locked`, `cargo build --locked` y **37 pruebas pertinentes**:

```powershell
cargo test --locked --test station_collision --test station_departure --test station --test ship_presentation --test ships --test ship_selection --test minigame_returns
```

Cubren superficies reales desde tres ejes, radios distintos, rotaciones/traslaciones/escalas, recuperación repetible, ausencia de barrera mundial, entrada segura para seis naves, despegue estable, cámara, nariz visual y controles existentes. Las antiguas pruebas que exigían colisión en espacio vacío dentro de la esfera se reemplazaron por verificaciones sobre celdas realmente ocupadas y el marco físico; no se eliminaron las comprobaciones de no atravesar la estación.

Se ejecutó el juego nativo con automatización de raylib: diagnóstico por arriba, lateral y recuperación; despegues con N‑1 y Smiling Friends; dos recorridos de tres regresos cada uno, con misma nave, cambio, cancelación y nuevos despegues. El recorrido de regreso parte de una posición de prueba cercana a la aproximación y luego usa navegación, E y selector reales. [Resultados](runtime-summary.json). Los logs resumidos están aquí y los completos en `target/station-diagnosis-runtime/`.

## Límites y reproducción

La colisión es una aproximación conservadora por superficies voxelizadas: puede detener la nave hasta una celda antes por eje, más la expansión de seguridad de su volumen. No es una colisión exacta por triángulos. El modo F3 permite ver ese ajuste. No hay una barrera global que deba eliminarse ni se desplazó la estación hacia las órbitas.

Raylib mantiene un aviso preexistente al convertir índices `u32` a `u16` en cinco primitivas de ventanas/avenidas/balizas de los brazos; el recurso de colisión lee los índices completos del original. Esta limitación del render de microdetalles no se corrigió modificando los modelos en esta tarea.

Queda la revisión manual con ratón, contactos oblicuos prolongados, turbo sostenido y aceptación del nuevo encuadre del N‑1. Las capturas y automatización no sustituyen esa valoración. Para revisar: despegar, pulsar F3, rodear los brazos, comparar rojo/cian al contactar una superficie visible, volver a la compuerta verde, entrar con E, cambiar de nave y repetir. Espacio conserva el ascenso.

Para regenerar el recurso derivado se necesita Python con numpy (solo durante generación):

```powershell
python tools/build_station_collision.py
```

Debe regenerarse si cambia el GLB fuente. El juego carga el JSON ya generado; no necesita numpy ni Blender.

Archivos modificados: `src/gameplay/station.rs`, `src/gameplay/mod.rs`, `src/scenes/solar_system.rs`, `src/ships/mod.rs`, `src/graphics/mod.rs`, `tests/station.rs`, `tests/station_departure.rs` y `tests/ship_presentation.rs`. Archivos nuevos: `src/gameplay/station_collision.rs`, `src/graphics/station_debug.rs`, `tests/station_collision.rs`, `tools/build_station_collision.py`, `tools/verify_station_diagnosis.py`, el JSON derivado y esta carpeta de evidencias. No se modificaron los archivos de presentación del hangar.
