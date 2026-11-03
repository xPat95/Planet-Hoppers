# Hangar del Presidium: interior, entrada y despegue

> Corrección de despegue: el punto cercano `puerta + 36` descrito abajo queda reservado a la aproximación. El vuelo comienza ahora fuera de los límites visuales y de colisión, contemplando nave y cámara, sin la antigua cámara frontal de salida. [Cálculo actual, pruebas y capturas](references/station-departure/README.md).

> Actualización visual posterior: la geometría procedural se sustituyó por el módulo Blender `citadel_hangar_interior.glb`, y se restauraron plataforma circular y giro. Las descripciones originales de geometría de este documento son históricas; los contratos de entrada, estado y despegue se conservan. [Informe visual actual y capturas](references/hangar/README.md).

## Arquitectura y continuidad espacial

El hangar anterior era un visor circular independiente, sin compuerta ni regreso funcional. El nuevo hangar es una **bahía complementaria dentro del grosor inferior del anillo aprobado**, no el GLB exterior colocado detrás de la nave. No se modifica el `.blend`, los GLB ni los modelos de las naves.

La fuente `tools/blender/spaceStation/crear_ciudadela.py::build_ring` define el anillo con sección achaflanada, radios interiores/exteriores de aproximadamente 29,5–40 y nervios/placas concéntricos. La exportación conserva su eje longitudinal X y se centra en su caja. El radio inferior usado por la bahía es 0,12 del largo ajustado: 168 unidades con el largo actual de 1400. La sala ocupa 36 × 16 × 56 unidades, dentro del volumen del blindaje de ese sector; su compuerta sobresale ligeramente de la cara frontal mediante un marco añadido.

`graphics::hangar_interior` construye suelo continuo, estacionamiento enrasado, paredes, troncos técnicos, pasarelas de servicio, techo completo con hombros achaflanados, nervios transversales y arcos concéntricos que continúan la curvatura del anillo. La compuerta abre hacia el espacio y las marcas del suelo conducen hasta ella. Plata, titanio azulado, grafito, luz hielo/cian y pequeños indicadores ámbar reproducen el lenguaje del recurso aprobado. Los cinco indicadores sobre la puerta representan los cinco acoplamientos de distritos.

La nave se apoya sobre el suelo usando sus límites ajustados. No gira automáticamente ni tiene un pedestal de exhibición. Se conserva `set_preview_angle` para las vistas automatizadas de Smiling Friends. Solo se mantiene cargada una nave. La escala de exhibición no modifica las dimensiones de vuelo.

## Una sola compuerta configurada

`config/station.json::dock` es el contrato compartido por interior, marco exterior, piloto, colisiones, interacción y despegue:

- `local_position: [0.505, -0.12, 0]`: coordenadas en ejes GLB centrados, como fracciones del largo del modelo. El punto está en la cara frontal del sector inferior del anillo, con un pequeño resalte para el marco exterior.
- `local_forward: [1, 0, 0]`, `local_up: [0, 1, 0]`: normal de salida y vertical de la bahía, antes de la rotación de la estación.
- `width: 20`, `height: 12`: abertura exterior e interior idéntica.
- `room_width: 36`, `room_height: 16`, `room_depth: 56`: dimensiones de la bahía complementaria.
- `departure_clearance: 36`, `approach_distance: 220`, `interaction_radius: 80`: separación al despegar, punto exterior de alineación y alcance final de entrada.

Si R es la rotación y P la posición de la estación, la puerta es `P + R(local_position × length)`. El despegue es `puerta + R(local_forward normalizado) × departure_clearance`; la nariz apunta en esa misma dirección. El suelo interior se interpreta en el marco de la puerta: centro de la bahía a media profundidad hacia dentro y medio alto de abertura por debajo de su centro. Con la configuración actual la puerta queda aproximadamente en **(0, 6032, 707)** y el despegue en **(0, 6032, 743)**.

El modelo exterior se ajusta primero sin rotación y luego se aplica R en su matriz de mundo. Así una rotación no cambia accidentalmente la escala ajustada ni separa la puerta del modelo. `graphics::station_dock` dibuja el marco, el umbral y las luces en ese mismo marco. No hay una segunda estación invisible.

La esfera conservadora de radio 860 se mantiene fuera de la entrada. Solo se permite un corredor rectangular alineado con la puerta; sus lados y fondo siguen bloqueados. El alcance de E se mide desde la puerta real y exige estar delante de ella y dentro de su ancho/alto. No se puede entrar desde la parte trasera, desde otra cara del anillo o desde el punto lejano de alineación.

## Ciclo de escenas y estado

`Menú → selección inicial → despegue → vuelo → E en compuerta → hangar → despegue`.

`solar_system::Session` permanece viva durante toda la visita. `Exit::Dock` libera la ventana y sus recursos gráficos antes de abrir el selector; conserva planetas, rotaciones, objetivos de minijuegos, últimas finalizaciones, nave inicial y estado de propiedad. No llama otra vez a `run_session` ni al menú principal. Al confirmar se aplica el modelo mediante el catálogo por identidad. Al cancelar se mantiene la nave anterior y se vuelve a la salida. Cerrar la ventana del hangar o Ctrl+Q termina la aplicación; Esc/Volver cancela la selección.

El modo `Docked` comparte el selector existente. Durante este prototipo la flota de seis naves permite cambios gratuitos en la sesión: al confirmar se incorpora únicamente la elegida a las disponibles y se activa, conservando la primera nave y las anteriores. Navegar o cancelar no modifica propiedad. `Initial` sigue desbloqueando solo una nave inicial y `Change` conserva sus restricciones de propiedad para la futura progresión. No se crean tokens, recompensas ni guardado.

El despegue restablece velocidad, arrastre, ladeo, vista de vuelo y cámara. Una breve vista de salida muestra la nave con la compuerta detrás y vuelve a la cámara de seguimiento. Las interacciones requieren liberar E/Enter/clic y una nueva pulsación; además hay un margen de 1,2 segundos antes de permitir el reingreso. No se entra por mantener E al confirmar o al regresar.

## Controles y pruebas manuales

```powershell
cargo run --locked
```

Jugar → recorrer las seis naves con flechas → Enter/Despegar. En el hangar **V** alterna nave, bahía completa y vista de salida. Esc/Volver regresa al menú en selección inicial; durante una visita a la estación vuelve al vuelo sin cambiar de nave.

En vuelo se conservan **Espacio para ascender**, W/S para avance/freno, Shift para turbo y **B para piloto**. Tab incluye la estación. Manteniendo B y W hacia ella se alcanza primero la alineación exterior; cerca de la boca, B limita la velocidad solicitada mientras se orienta para evitar pasar de largo con turbo. Continuar hasta ver **[E] Entrar a la estación espacial**, soltar y pulsar E. El selector abre mostrando la nave utilizada; elegir otra y despegar. El piloto no modifica el control manual cuando se suelta B.

## Verificación automatizada y capturas

```powershell
cargo check --locked
cargo build --locked --bins
cargo test --locked --test station --test ship_selection --test ships --test minigame_returns
cargo run --locked -- --smoke-docking --ship minecraft
cargo run --locked -- --smoke-returns --ship starwars
```

`--smoke-docking` recorre el menú y las seis exhibiciones; después realiza tres aproximaciones con los eventos nativos W/B/E: visita conservando la nave, visita cambiándola y visita cancelando. Comprueba despegue seguro/orientado, modelo cargado por identidad, conservación de órbitas/rotaciones y estado, selección al entrar, cambio confirmado y cancelación. La aproximación comienza desde un punto exterior preparado por el recorrido; no simula un viaje manual completo desde un planeta. Mantiene E pulsada al inicio del retorno para comprobar que no produce un ingreso automático.

Capturas en `target/front-end/`: `hangar-complete.png` (vista general), `hangar-approach.png`, `hangar-arrival.png`, `hangar-transform.png`, `hangar-close-up.png`, `hangar-<id>.png`, `docked-entry.png` y `docked-selected.png`. La cámara se acerca automáticamente desde la vista general; al cambiar entre naves, los encuadres especiales de Star Wars y Minecraft se interpolan para evitar saltos de la estación. En `target/solarsystem/`: `dock-launch-0.png` a `dock-launch-3.png`, `dock-approach-0.png` a `dock-approach-2.png`. Informe de recorrido: `target/hangar-cycle-smoke.log`.

Las pruebas de estación verifican transformaciones con varias rotaciones/traslaciones, salida segura, restricciones de entrada, corredor de colisión y aproximación desde el inicio solar en normal/turbo. Las de selección comprueban la sesión, identidades y propiedad; las existentes cubren vuelo y retorno de South Park.

Resultado de esta implementación: `cargo check --locked` y `cargo build --locked --bins` correctos; las 25 pruebas pertinentes de estación, selección, vuelo y regreso aprobaron. La suite completa con `--no-fail-fast` dio **163 aprobadas y una falla previa** de Smiling Friends. El recorrido gráfico de tres visitas produjo `DOCKING_SMOKE_PASS`; la regresión con cuatro visitas a South Park produjo `VISIT_SMOKE_PASS visitas=4` conservando la nave. Se revisaron las seis capturas de selección, la bahía completa, la salida, despegues y aproximaciones. `cargo fmt --check` sigue mostrando diferencias globales; se verificó por separado el formato de los módulos nuevos y de la geometría, sin reformatear masivamente el código heredado.

Archivos de implementación: `config/station.json`, `src/gameplay/station.rs`, `src/gameplay/selection.rs`, `src/graphics/hangar.rs`, `src/graphics/hangar_interior.rs`, `src/graphics/station_dock.rs`, `src/graphics/mod.rs`, `src/scenes/front_end.rs`, `src/scenes/solar_system.rs`, `tests/station.rs` y `tests/ship_selection.rs`. Documentación: este archivo, `docs/hangar.md`, `docs/solar-system.md` y el README del recurso de estación.

## Límites intencionales

- La bahía es geometría complementaria del motor, no una modificación del modelo aprobado ni una reconstrucción completa del interior de la Ciudadela.
- La compuerta exterior tiene un receso oscuro y marco físico; el cambio a la sala usa fundido. No hay vuelo continuo a través de un portal ni cinemática de atraque.
- No hay peatón, estación explorable, tokens ni persistencia. El futuro interior artístico puede sustituir el entorno manteniendo el contrato de puerta y escenas.
- Quedan para revisión humana la respuesta a teclado/mouse físicos y una medición prolongada de rendimiento. Las capturas y eventos automatizados no sustituyen esa evaluación.
- Se conserva la falla previa de `tests/solar_system.rs::la_proximidad_conserva_la_identidad_y_el_minijuego_en_las_nuevas_orbitas`: espera `None` para Smiling Friends aunque `main` ya habilitó ese minijuego. También existen diferencias globales de formato previas. No se corrigen masivamente en esta tarea.
