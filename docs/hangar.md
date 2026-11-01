# Menú principal y selección de naves

## Entrada normal y acceso de desarrollo

```powershell
cargo run --locked                         # menú principal
cargo run --locked --bin solarsystem       # sistema solar directo, catálogo de desarrollo
cargo run --locked --bin southpark         # South Park independiente
```

El ejecutable predeterminado es `planet-hoppers`. El flujo del jugador es **Menú principal → Nueva partida → Hangar → Comenzar aventura → Sistema solar**. Controles muestra la ayuda y Salir cierra la aplicación. No hay guardado funcional: se omite Continuar y el estado dura únicamente durante la sesión.

Las escenas se ejecutan secuencialmente en el mismo proceso. El menú y el hangar comparten una ventana; esta libera sus recursos antes de abrir el sistema solar. Se conserva el contrato existente de ventanas secuenciales con South Park.

## Catálogo e identidad

`config/ship-selection.json` establece seis opciones en orden orbital y las vincula al catálogo de modelos de `config/ships.json`:

| Identidad | Franquicia mostrada | Recurso |
| --- | --- | --- |
| `halloween` | Calaca | Placeholder geométrico identificable; modelo de Camila pendiente |
| `starwars` | Estrella de la Muerte | N-1 del Mandaloriano |
| `smilingfriends` | Smiling Friends | Nave de Smiling Friends |
| `minecraft` | Minecraft | Barco del End |
| `regularshow` | Un Show Más | Carrito de golf |
| `southpark` | South Park | Nave de Butters |

Las seis están disponibles gratuitamente en selección inicial. Star Wars utiliza el N-1 del Mandaloriano; el N-1 de Naboo se conserva en el catálogo de desarrollo. Para incorporar la nave final de Calaca se agrega su modelo a `config/ships.json` y se asigna su identificador a `ship_id`; la identidad `halloween` y el estado de la partida permanecen iguales. El placeholder se muestra tanto en el hangar como en vuelo, con tamaño de nave de 4 unidades.

Las flechas de pantalla o izquierda/derecha recorren el catálogo de forma circular. Enter o el botón confirman; Volver/Esc cancela sin iniciar ni modificar una partida. Solo se retiene un modelo de exhibición: se libera antes de cargar el siguiente. Si falla un recurso, la interfaz muestra un placeholder con la identidad correspondiente.

## Estado y selección compartida

`gameplay::selection` contiene las reglas, sin renderizado:

- `ShipCatalog` valida las seis identidades y resuelve sus modelos.
- `GameState` registra nave inicial, nave activa y conjunto de desbloqueadas. Una partida nueva desbloquea exactamente una y no crea tokens.
- `ShipSelection` comparte navegación y confirmación entre `Initial` y `Change`.
- `Initial` crea un estado nuevo solo al confirmar; todas las opciones son elegibles.
- `Change` permite visualizar todas, pero únicamente confirma una desbloqueada. Devuelve una copia actualizada conservando nave inicial y propiedad; no crea otra partida.

El punto de entrada `scenes::front_end::select(mode, game)` reutiliza la misma vista para una futura visita a la estación. Al cancelar devuelve `None`. Al cambiar devuelve `SelectionTransition::ChangedShip`; el futuro anfitrión debe aplicar ese estado a su sesión solar existente, sin volver a ejecutar el inicio del sistema solar.

Al confirmar una selección inicial se devuelve `SelectionTransition::StartAdventure(game)`. `front_end::run` entrega ese estado a `solar_system::run_with_game`. Allí se resuelve el modelo por identidad y se fija la nave activa antes de crear la sesión espacial. El catálogo no sustituye la selección por `active_ship` predeterminada del archivo. El estado permanece dentro de la sesión solar durante las visitas a South Park y se devuelve al terminar.

El acceso directo de desarrollo mantiene N para recorrer todos los modelos. En una aventura, N únicamente recorre las naves desbloqueadas; al comienzo solo está disponible la elegida. No se añaden controles de estación ni un sistema de progreso.

## Hangar provisional y estación de Norman

`graphics::hangar` contiene únicamente la presentación:

- `HangarStyle` centraliza tamaño de exhibición (6 unidades), plataforma (radio 4,5), giro (10°/s), y transformación del futuro modelo de estación.
- `HangarCamera` encuadra la esfera que contiene la nave en cualquier ángulo de giro, considerando las áreas ocupadas por encabezado, pie y flechas. Conserva FOV de 42°.
- `Hangar` carga materiales/texturas mediante `SpaceModel` y utiliza la iluminación existente con una luz local y ambiente de 0,55. No cambia ningún shader solar o espacial.
- El fondo provisional consiste en suelo, dos paredes y unas tiras luminosas. La plataforma y la nave giran juntas; la cámara permanece estable.

Para integrar el GLB definitivo de Norman se configura `HangarStyle.station_model`, junto con `station_length`, `station_position` y `station_rotation`. El adaptador `draw_backdrop` sustituye el interior provisional por ese recurso; la selección, los desbloqueos, la cámara y las transiciones no dependen de la geometría de las paredes. El ajuste de posición debe alinearse con la plataforma tras inspeccionar el modelo definitivo.

La escala de exhibición es una transformación local del visor. El vuelo sigue utilizando los largos originales de 4–5 unidades, su radio de colisión y las velocidades existentes. No se modifican mallas de Blender, planetas, órbitas, skybox ni shaders del Sol.

## Puntos de extensión

- La abducción inicial se insertará al consumir `StartAdventure`, antes de `run_with_game`.
- Un futuro consumidor de progreso decidirá cuándo invocar `GameState::unlock`, con las reglas de tokens y sus posibles cinemáticas. El mutador actual solo actualiza propiedad y no concede recompensas.
- La entrada física a la estación invocará el selector en modo `Change` y aplicará `ChangedShip` a la sesión solar existente.
- No se implementan guardado, Continuar, tokens, recompensas, cinemáticas ni acceso a la estación definitiva.

## Verificación reproducible

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
