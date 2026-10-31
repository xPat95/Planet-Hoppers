# SAVE KENNY! — fase 4A: tres amenazas independientes

## Ejecutar y activar eventos

```powershell
cargo run --locked
```

El paseo libre y el diálogo de fase 3 siguen disponibles. Las amenazas se prueban mediante teclas de desarrollo cuando el diálogo está cerrado:

| Control | Acción |
| --- | --- |
| F5 | Preparar autobús |
| F6 | Preparar OVNI |
| F7 | Preparar alcantarilla |
| F8 | Reiniciar la amenaza seleccionada |
| F9 | Retirar el evento y volver al paseo libre |
| E | Ejecutar la interacción apuntada |
| WASD / mouse / Shift | Caminar, mirar, correr |
| Escape | Liberar/recuperar cursor; cerrar diálogo cuando corresponda |
| F1 / F2 / F3 / F12 | Ayuda, colisionadores, indicador de Kenny, captura |
| Ctrl+Q o cerrar ventana | Salir |

En laptops puede requerirse Fn para las teclas F. También se puede iniciar directamente un evento, retirando la utilería técnica de fase 2:

```powershell
cargo run --locked -- --threat bus
cargo run --locked -- --threat ufo
cargo run --locked -- --threat sewer
```

Las advertencias y el estado aparecen arriba. Solo hay un evento en curso. Pulsar otra tecla de amenaza durante preparación, actividad o resolución muestra un aviso y conserva la actual. F8 reinicia; F9 limpia los objetivos/colisionadores temporales y coloca a Kenny en una posición segura próxima a la que tenía antes del evento. Si el jugador ocupa una aparición requerida, se rechaza el cambio con un mensaje: apartarse y repetir.

La fase actual utiliza preparación explícita de desarrollo: Kenny se coloca en la zona del evento, no necesita recorrer la ciudad desde una posición distante. El jugador conserva su posición. No se seleccionan eventos aleatoriamente y aceptar el diálogo de fase 3 sigue mostrando solo su confirmación preparatoria.

## Zonas y tiempos

Se usan los mismos metros y ejes de la fase 3: Y arriba, suelo en Y=0. Los eventos se sitúan en North Avenue, Z=-42.9, cerca de la aparición del jugador en X=-43.2. No hay que cruzar la ciudad.

`config/southpark-threats.json` configura preparación, duración, alcance y límite de elevación. Se lee al cargar el escenario; reiniciar la aplicación para aplicar cambios. Los valores iniciales son:

| Evento | Preparación | Ventana activa | Posición inicial de Kenny |
| --- | ---: | ---: | --- |
| Autobús | 3 s | hasta 16 s/contacto | (-24, 0, -42.9) |
| OVNI | 3 s | 18 s | (-28, 0, -42.9) |
| Alcantarilla | 3 s | 18 s | (-26, 0, -42.9) |

La configuración rechaza números inválidos y ventanas excesivamente cortas. Las trayectorias se parametrizan por duración: ampliar el tiempo reduce la velocidad de aproximación/elevación sin desplazar sus extremos. Se utiliza dt con el límite de 0.1 s heredado de fase 2; las pausas largas no producen saltos de simulación. Escape libera el cursor pero no pausa el evento.

El código verifica el recorrido completo de Kenny y el barrido de la caja del autobús contra los obstáculos de la ciudad. Los accesos a los mecanismos también se comprueban. La ruta del autobús es recta y permanece dentro de la calle de 6 m; su caja conservadora tiene 6.2 × 2.7 × 2.44 m. Si avanza sobre el volumen del jugador, lo aparta a un punto lateral comprobado para no dejar al FPS atrapado dentro del modelo. No hay daño al jugador ni simulación física de vehículos.

## Autobús escolar

El autobús amarillo tiene ventanas oscuras, parabrisas, ruedas, faros, bandas y rótulo. Comienza a 48 m por delante de la posición inicial de Kenny y avanza hacia -X. Kenny recorre 8 m de manera controlada mientras el vehículo lo alcanza. La velocidad inicial del autobús es aproximadamente 3.3 m/s.

El panel rojo está en (-33, 0, -40.9), a unos 10 m del punto inicial del jugador. Hay que desplazarse y orientar la cámara: el alcance es 4.5 m, y no se puede frenar presionando E desde la aparición. Apuntar al panel muestra **[E] Activar freno**. La interacción compartida respeta obstáculos.

Activarlo durante la fase activa congela la posición del autobús, registra `Survived` y permite a Kenny continuar un paseo válido. Si la caja del autobús contacta con Kenny antes, registra `Died`; su sprite se inclina/comprime brevemente y se oculta. F8 restaura posiciones, resultado y preparación.

## OVNI

El platillo tiene diámetro 5.44 m y oscila suavemente alrededor de Y=6.2 m. Sus tres luces están bajo el borde, separadas 120°, y pueden apuntarse desde el suelo. El alcance específico es 14 m; las demás interacciones conservan su alcance habitual.

Durante la actividad se ve un rayo tractor translúcido y Kenny asciende hasta 4.5 m. Cada luz es un objetivo independiente. Después de la caja amplia se prueba la esfera real de 0.25 m, evitando aciertos en esquinas vacías. Aparece **[E] Desactivar luz N**. Una luz apagada se oscurece, desaparece de los objetivos y no vuelve a contarse. El indicador temporal muestra el progreso 0/3 a 3/3.

Apagar las tres entra en resolución: se corta el rayo y Kenny desciende a 2 m/s. Solo al llegar al suelo se registra `Survived` y vuelve a caminar. Si alcanza el límite antes, se registra `Died`, continúa una breve elevación y desaparece. F8 reactiva las tres luces y lo devuelve al suelo. No se implementa salto, nave pilotable ni extraterrestres.

## Alcantarilla

La abertura está en (-12, 0, -42.9); la tapa metálica comienza separada en (-32, 0, -40.9). Kenny camina desde X=-26 hacia el agujero durante la ventana de 18 s.

Primero apuntar hacia abajo a la tapa y pulsar **[E] Recoger tapa**. Desaparece del suelo y aparece el indicador **Llevas una tapa**. Entonces se habilita el objetivo de la abertura: acercarse, apuntarla y pulsar **[E] Colocar tapa**. No hay opción de colocarla antes de recogerla. Hay margen para caminar entre ambos puntos con la velocidad FPS normal.

Colocarla reproduce un breve cierre y registra `Survived`. Si Kenny llega con la abertura descubierta, se detiene, cae verticalmente hasta unos 3 m bajo el suelo y se oculta; se registra `Died`. F8 retira el cierre y cualquier estado de transporte y devuelve la tapa a su lugar. Es una amenaza a Kenny: el suelo FPS sigue siendo plano, sin caída física del jugador ni inventario general.

## Arquitectura y resultados

`src/minigames/southpark/threats.rs` contiene identificadores, configuración y un único `ThreatDirector`. Sus estados son `Inactive`, `Preparing`, `Active`, `Resolving`, `Succeeded` y `Failed`. `kind` selecciona el evento; `kenny_id` enlaza al actor existente. Tiene métodos de inicio, actualización, interacción, reinicio y limpieza. La referencia mutable a Kenny se entrega desde la escena, evitando referencias internas inválidas.

Durante el evento se pausa el paseo libre y se aplican poses controladas en rutas previamente comprobadas. El cuerpo de Kenny ahora utiliza también su altura real durante elevación/caída. El billboard y su transparencia se conservan. Al terminar con éxito se restaura el paseo; un fracaso mantiene al actor detenido/oculto hasta reiniciar o salir de la prueba.

Los objetivos `threat:*` se registran en la lista existente y pasan por `select_target_with`, incluido el control de oclusión. No hay un raycaster nuevo. Los objetivos de Kenny se deshabilitan durante el peligro para que abrir su diálogo no permita detenerlo. Las luces desaparecen individualmente y los objetivos de tapa se sustituyen en secuencia.

`result: Option<ThreatResult>` expone `Survived` o `Died` para fase 5. Se publica una vez al alcanzar el resultado y persiste hasta reinicio/nuevo evento. Los contactos de Kenny también notifican el tipo de peligro; se limpian al reiniciar. Se muestra confirmación de supervivencia o un mensaje temporal de muerte. No se descuentan vidas, suman salvadas, entregan tokens ni concluye una partida.

Para fase 4B, añadir variantes de piano y muñeco de nieve al despacho del coordinador, sus recursos, zonas y reglas. El flujo de estados, exclusión de eventos simultáneos, raycasting, resultados, reinicio y vuelta a exploración permanecen compartidos. No existen aún modelos ni lógica de esas dos amenazas.

## Modelos y generación

El [README de recursos](../../assets/models/minigames/southpark/threats/README.md) describe las escenas editables y ejes. El script `tools/blender/southpark/build_threat_assets.py` crea **southpark_threats.blend** y cinco GLB: bus, ufo, sewer, lid y brake_panel. El archivo editable contiene cuatro escenas, con la tapa separada dentro de la escena de alcantarilla. No abre ni guarda sobre la ciudad original.

Los materiales son colores planos opacos, sin texturas ni fuentes externas. Las tres bombillas y el cono translúcido se dibujan como primitivas sencillas de raylib para variar color/altura sin duplicar el modelo ni crear recursos por cuadro. Los cuerpos GLB, materiales y shaders se cargan una vez. En total, las cinco exportaciones tienen 6088 triángulos.

Se reutiliza el exportador estático existente. El generador limpia selecciones de todas las escenas antes de exportar para evitar mezclar recursos o incluir el cubo predeterminado. `asset-audit.json` registra conteos y límites; Rust compara triángulos y las seis coordenadas de límites de cada modelo contra esa auditoría. También se verifican los colores con el cargador de fase 2. Una exportación incompleta o mezclada falla antes de jugar.

## Verificación

```powershell
cargo check --locked
cargo build --locked
cargo fmt --all -- --check
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
cargo run --locked -- --smoke-threats
cargo run --locked -- --smoke
cargo run --locked -- --smoke-kenny
```

Las pruebas incluyen 14 nuevas de amenazas y las 34 anteriores: **48 en total**. Cubren los corredores reales, un solo evento activo, bloqueo de interacciones prematuras, frenado, contacto, movimiento a distintos FPS, tres luces únicas, precisión de esferas y superficies circulares, ascenso/descenso, recogida/colocación, caída, resultados, reinicios, limpieza de objetivos y recuperación del paseo. Se prueba alcance/oclusión del panel y que el autobús no deje el volumen FPS encerrado.

`--smoke-threats` abre un contexto OpenGL real, carga y audita los cinco GLB y ejecuta seis escenarios: éxito y fracaso de cada amenaza. En los éxitos, E se inyecta mediante eventos de raylib y pasa por los mismos controles/raycasting de la aplicación normal. Verifica 1 acción de freno, 3 luces y 2 acciones de tapa; también reinicia cada caso y vuelve a explorar sin objetivos o colisionadores sobrantes.

La prueba gráfica posiciona y orienta automáticamente al jugador para aislar las interacciones y usa cámaras de contexto para las capturas. No pretende comprobar un recorrido manual ni la dificultad final. Escribe [informe](../references/southpark/phase4a/runtime-validation.json), [autobús](../references/southpark/phase4a/bus.png), [OVNI](../references/southpark/phase4a/ufo.png) y [alcantarilla](../references/southpark/phase4a/sewer.png). Logs y evidencia de regresión FPS quedan en `target/`; `--smoke-kenny` usa ahora `target/southpark-kenny-smoke/` para preservar las capturas históricas de fase 3.

Pendiente de validación manual: comodidad para encontrar/apuntar mecanismos, tiempos en la laptop real, teclas F/Fn, cambios de foco y sesiones largas. El modo libre no acerca automáticamente al jugador si activa un evento desde lejos: para probar sin caminar toda la ciudad, iniciarlo desde el arranque con `--threat` o hacerlo cerca de la aparición. Las colisiones son cajas conservadoras y el apartamiento lateral del jugador es una solución de desarrollo, no física de vehículos. No se ha perfilado el rendimiento de la partida completa.

## Archivos y preservación

Nuevos: `config/southpark-threats.json`, `src/minigames/southpark/threats.rs`, `src/graphics/threats.rs`, `src/scenes/threat_smoke.rs`, `tests/southpark_threats.rs`, el generador Blender, los recursos de `assets/models/minigames/southpark/threats/`, esta guía y cuatro evidencias en `docs/references/southpark/phase4a/`.

Modificados: declaraciones de módulos gráficos/escenas, `src/graphics/resources.rs` para dibujar modelos con transformación, `kenny.rs` para poses controladas y cuerpo elevado, `southpark/mod.rs` para coordinar amenazas, y `southpark_dev.rs` para teclas, objetivos, renderizado y prueba gráfica. Los README y la arquitectura enlazan esta fase. No hay nuevas dependencias.

El planeta, ciudad Blender/GLB/manifiesto, PNG de Kenny, naves y recursos de fases anteriores se conservan. No hay música, diálogos nuevos de otros personajes, vidas, cinco salvadas, selección aleatoria ni integración solar. No se ejecuta commit, push, cambio de rama, merge o rebase.
