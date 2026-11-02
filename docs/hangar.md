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

## Hangar circular inspirado en la estación

`graphics::hangar` contiene únicamente la presentación:

- `HangarStyle` centraliza tamaño de exhibición (6 unidades), plataforma (radio 4,5), giro (10°/s), posición de la luz local, ambiente (0,55), estilo del interior y transformación de un eventual modelo interior.
- `HangarCamera` conserva el encuadre de la esfera que contiene la nave en cualquier ángulo de giro, considerando encabezado, pie y flechas. Dirección, distancia mínima (11), margen (1,08) y FOV (42°) quedan configurables por separado.
- `graphics::hangar_interior` construye solamente el entorno visual. `InteriorStyle` centraliza radio (12), altura (6,8), 60 segmentos, cinco accesos radiales y paleta de grafito, titanio azulado, plata, cian y hielo.
- La referencia es el anillo del Presidium y los cinco acoplamientos del [modelo aprobado de la Ciudadela](../assets/models/environment/spaceStation/README.md), especialmente [Ciudadela_Anillo.png](references/spaceStation/Ciudadela_Anillo.png). Se traducen sus placas segmentadas, bandas concéntricas, nervios y pequeñas ventanas a una sala circular. Los accesos usan marcos sucesivos y guías de suelo para sugerir profundidad hacia los brazos.
- El lado cercano se presenta como un corte arquitectónico: omite pared y franja de techo frente a la cámara para que nunca oculten la nave. El suelo circular permanece completo. La parte superior es una corona abierta; no se pretende construir una estación visitable.
- El entorno usa primitivas batched de raylib con colores de caras preajustados y pequeñas franjas luminosas sin iluminación dinámica adicional, bloom, texturas nuevas ni sombras. La nave conserva sus materiales/texturas y el shader existente con luz local. La plataforma tiene tapa de grafito y un borde cian discreto; sus marcas y la nave siguen girando juntas.
- Las pequeñas ventanas superiores repiten las bandas habitadas del exterior aprobado: luz hielo y algunos acentos ámbar sobre el titanio frío. La estación real ya aparece fija sobre el sistema solar; su [posición, aproximación y controles](solar-system.md#estación-espacial-fija) se configuran por separado del visor. La interacción exterior todavía no abre el selector.

Para integrar un GLB interior definitivo se configura `HangarStyle.station_model`, junto con `station_length`, `station_position` y `station_rotation`. El adaptador `draw_backdrop` sustituye el entorno procedural por ese recurso; conserva plataforma, nave, selección, desbloqueos, cámara y transiciones. El GLB exterior aprobado no se carga como habitación: contiene el conjunto completo y no define un espacio interior preparado para exhibición. No se modifica su fuente Blender ni sus exportaciones.

Se puede expandir o reemplazar únicamente `InteriorStyle::draw` para incorporar pasarelas, techos o módulos interiores completos. Ambos modos del selector comparten este mismo entorno. Quedan pendientes el modelo interior final, materiales PBR específicos si se desean, sombras de contacto y una medición sostenida de rendimiento en el hardware objetivo; no se necesitan para el flujo actual.

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

Verificación del rediseño circular: `cargo build --locked --bins` y la suite completa de `cargo test --locked` pasaron. Se ejecutó `planet-hoppers --smoke --ship minecraft`: menú, Controles, regreso, seis exhibiciones, ambas flechas, cancelación, nueva selección y arranque del sistema solar con Minecraft; los tres informes `SELECTION_SMOKE_PASS`, `LAUNCH_SMOKE_PASS` y `FRONT_END_SMOKE_PASS` resultaron correctos. Se revisaron las capturas de las seis naves, incluida Calaca provisional, comprobando encuadre y legibilidad de UI. Evidencia: `target/front-end/hangar-<id>.png`, `target/front-end/selection-validation.json` y `target/hangar-redesign-smoke.log`. No se hizo una medición sostenida de FPS ni una partida manual en esta revisión.
