# Menú principal y selección de naves

> Presentación actual: cámara de la vista general aprobada, iluminación de Cycles horneada y nebulosa original en la salida. [Comparación Blender/juego, capturas y verificaciones](references/hangar/presentation/README.md). No se rediseñó el modelo ni se modificaron plataforma, naves o mecánicas.

> Estado actual — Fase 2: el selector usa `citadel_hangar_reconstructed.glb`, exportado del Blender aprobado. [Integración, capturas reales, ajuste de cámara por clipping y verificación funcional](references/hangar/integration/README.md). Las descripciones de implementaciones anteriores que siguen abajo son históricas.

> Actualización visual: el entorno procedural descrito históricamente abajo fue sustituido por `citadel_hangar_interior.glb`, construido en Blender desde la referencia aprobada. La plataforma circular de radio 4.5 y el giro de 10°/s están restaurados. [Informe actual con renders, capturas reales y validaciones](references/hangar/README.md).

## Entrada normal y acceso de desarrollo

```powershell
cargo run --locked                         # menú principal
cargo run --locked --bin solarsystem       # sistema solar directo, catálogo de desarrollo
cargo run --locked --bin southpark         # South Park independiente
```

El ejecutable predeterminado es `planet-hoppers`. El flujo del jugador es **Menú principal → Nueva partida → Hangar → Comenzar aventura → Sistema solar**. Controles muestra la ayuda y Salir cierra la aplicación. No hay guardado funcional: se omite Continuar y el estado dura únicamente durante la sesión.

Las escenas se ejecutan secuencialmente en el mismo proceso. El menú y el hangar comparten una ventana; esta libera sus recursos antes de abrir el sistema solar. Se conserva el contrato existente de ventanas secuenciales con South Park.

## Catálogo e identidad

`config/ship-selection.json` establece seis opciones en orden de presentación en el hangar, con Halloween al final, y las vincula al catálogo de modelos de `config/ships.json`:

| Identidad | Franquicia mostrada | Recurso |
| --- | --- | --- |
| `starwars` | Estrella de la Muerte | N-1 del Mandaloriano |
| `smilingfriends` | Smiling Friends | Nave de Pim y Charlie |
| `minecraft` | Minecraft | Barco del End |
| `regularshow` | Un Show Más | Carrito de golf |
| `southpark` | South Park | Nave de Butters |
| `halloween` | Calaca | Bruja en escoba |

Las seis están disponibles gratuitamente en selección inicial. Star Wars utiliza el N-1 del Mandaloriano; el N-1 de Naboo se conserva en el catálogo de desarrollo. Para incorporar la nave final de Calaca se agrega su modelo a `config/ships.json` y se asigna su identificador a `ship_id`; la identidad `halloween` y el estado de la partida permanecen iguales. El placeholder se muestra tanto en el hangar como en vuelo, con tamaño de nave de 4 unidades.

Las flechas de pantalla o izquierda/derecha recorren el catálogo de forma circular. Enter o el botón confirman; Volver/Esc cancela sin iniciar ni modificar una partida. Solo se retiene un modelo de exhibición: se libera antes de cargar el siguiente. Si falla un recurso, la interfaz muestra un placeholder con la identidad correspondiente.

## Estado y selección compartida

`gameplay::selection` contiene las reglas, sin renderizado:

- `ShipCatalog` valida las seis identidades y resuelve sus modelos.
- `GameState` registra nave inicial, nave activa y conjunto de desbloqueadas. Una partida nueva desbloquea exactamente una y no crea tokens.
- `ShipSelection` comparte navegación y confirmación entre `Initial` y `Change`.
- `Initial` crea un estado nuevo solo al confirmar; todas las opciones son elegibles.
- `Change` permite visualizar todas, pero únicamente confirma una desbloqueada. Devuelve una copia actualizada conservando nave inicial y propiedad; no crea otra partida.

El punto de entrada `scenes::front_end::select(mode, game)` comparte la vista con `front_end::dock`, ya conectado a la sesión solar. Al cancelar devuelve `None`. Al cambiar devuelve `SelectionTransition::ChangedShip`; el anfitrión aplica ese estado sin volver a ejecutar el inicio del sistema solar.

Al confirmar una selección inicial se devuelve `SelectionTransition::StartAdventure(game)`. `front_end::run` entrega ese estado a `solar_system::run_with_game`. Allí se resuelve el modelo por identidad y se fija la nave activa antes de crear la sesión espacial. El catálogo no sustituye la selección por `active_ship` predeterminada del archivo. El estado permanece dentro de la sesión solar durante las visitas a South Park y se devuelve al terminar.

El acceso directo de desarrollo mantiene N para recorrer todos los modelos. En una aventura, N únicamente recorre las naves disponibles; al comienzo solo está disponible la elegida. El cambio libre en el hangar incorpora la nave confirmada a la sesión, sin tokens ni guardado.

## Interior del hangar y ciclo con la estación

El hangar ahora es una bahía complementaria en el sector inferior del anillo de la Ciudadela. Tiene suelo de estacionamiento, paredes, techo completo achaflanado, arcos concéntricos, pasarelas técnicas y una abertura de salida. La nave queda apoyada sobre el suelo y no gira automáticamente. No se usa el modelo exterior como fondo ni se modifica su fuente.

La compuerta exterior, el interior y el punto de despegue comparten `config/station.json::dock`. La entrada desde el vuelo ya abre el selector; al salir se reutiliza la misma sesión solar. El modo `Docked` permite probar las seis naves mediante cambios gratuitos durante la sesión, sin tokens ni guardado; `Initial` y `Change` conservan sus contratos anteriores.

`HangarStyle` conserva luz local/ambiente y parámetros de cámara; las dimensiones interiores proceden de la compuerta configurada. V alterna la nave, la bahía completa y la vista de salida. Se conserva el adaptador de un futuro GLB interior, independiente del catálogo y de las transiciones.

Ver [arquitectura, transformaciones, controles, recorridos y capturas](station-hangar-cycle.md). Las versiones anteriores del entorno circular y su verificación quedan reemplazadas por este ciclo funcional.

## Puntos de extensión

- La abducción inicial se insertará al consumir `StartAdventure`, antes de `run_with_game`.
- Un futuro consumidor de progreso decidirá cuándo invocar `GameState::unlock`, con las reglas de tokens y sus posibles cinemáticas. El mutador actual solo actualiza propiedad y no concede recompensas.
- La entrada física al hangar ya invoca el selector compartido y aplica `ChangedShip` a la sesión solar existente. El modo restringido `Change` queda disponible para futuras reglas de progreso.
- No se implementan guardado, Continuar, tokens, recompensas, cinemáticas ni una estación recorrible a pie.

## Verificación del visor anterior (histórica)

La verificación actual del ciclo completo se documenta en [Hangar del Presidium](station-hangar-cycle.md). Los resultados siguientes corresponden a la implementación anterior, antes de integrar los cambios de `main` y el ciclo de entrada/despegue.

```powershell
cargo build --locked --bins
cargo test --locked
cargo run --locked -- --smoke --ship minecraft
cargo run --locked -- --smoke --ship halloween
cargo run --locked -- --smoke-returns --ship halloween
```

La prueba del menú inyecta eventos nativos de teclado/mouse: abre Controles, vuelve, entra al hangar, muestra las seis opciones, usa ambas flechas, vuelve al menú, entra otra vez y confirma la identidad solicitada. Comprueba la carga de los cinco modelos disponibles y el placeholder de Calaca. Después abre la vista de vuelo y comprueba que la nave activa y la inicial coinciden, y que solo hay una desbloqueada. `--smoke-returns` continúa con las cuatro visitas existentes a South Park y verifica que mantienen esa elección.

Capturas e informe de selección: `target/front-end/`. Capturas de arranque: `target/solarsystem/launch-<id>.png`. Las pruebas de lógica cubren todas las selecciones, bloqueo de confirmación en modo cambio, conservación del estado al navegar/cancelar, encuadre y ausencia de cambios de escala en vuelo. Las pruebas físicas existentes siguen cubriendo turbo, colisiones, movimiento y piloto automático.

Validación completada: compilación de todos los binarios y 116 pruebas aprobadas. Se ejecutó el flujo completo de menú y arranque con cada una de las seis identidades. También pasó el flujo desde el menú con Calaca y las cuatro visitas a South Park (victoria, derrota y dos abandonos), manteniendo Calaca como nave activa e inicial y una sola desbloqueada. Se revisaron capturas de las seis exhibiciones y se corrigió el encuadre del Barco del End para que el pie de interfaz no lo tape.

La validación automatizada no sustituye una sesión manual con teclado/mouse físicos, una evaluación sostenida de FPS ni la revisión del futuro modelo de estación. Persiste el aviso conocido de raylib sobre los índices del planeta de Star Wars; no se modificó su cargador ni ese recurso.

Verificación del rediseño circular: `cargo build --locked --bins` y la suite completa de `cargo test --locked` pasaron. Se ejecutó `planet-hoppers --smoke --ship minecraft`: menú, Controles, regreso, seis exhibiciones, ambas flechas, cancelación, nueva selección y arranque del sistema solar con Minecraft; los tres informes `SELECTION_SMOKE_PASS`, `LAUNCH_SMOKE_PASS` y `FRONT_END_SMOKE_PASS` resultaron correctos. Se revisaron las capturas de las seis naves, incluida Calaca provisional, comprobando encuadre y legibilidad de UI. Evidencia: `target/front-end/hangar-<id>.png`, `target/front-end/selection-validation.json` y `target/hangar-redesign-smoke.log`. No se hizo una medición sostenida de FPS ni una partida manual en esta revisión.
