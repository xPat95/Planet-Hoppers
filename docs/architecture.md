# Arquitectura de Planet Hoppers

## Alcance inicial

Esta arquitectura organiza el trabajo sin elegir lenguaje, motor gráfico, librerías ni formatos de configuración. Por ahora solo existen documentación y carpetas reservadas; las responsabilidades siguientes guiarán la implementación futura.

## Recursos y configuración

| Carpeta | Responsabilidad |
| --- | --- |
| `assets/models/planets/` | Modelos de los seis planetas, separados por franquicia. |
| `assets/models/ships/` | Modelos de las seis naves, separados por franquicia. |
| `assets/models/characters/` | Modelos de los personajes de cada franquicia para las animaciones inicial y final. |
| `assets/models/environment/` | Modelos de escenarios y elementos ambientales compartidos. |
| `assets/models/environment/spaceStation/` | Modelos 3D y recursos geométricos asociados con la estación espacial. |
| `assets/textures/planets/` | Texturas de planetas. |
| `assets/textures/ships/` | Texturas de naves. |
| `assets/textures/characters/` | Texturas de personajes. |
| `assets/textures/minigames/` | Texturas de los escenarios y elementos de minijuegos. |
| `assets/textures/skybox/` | Texturas del fondo espacial. |
| `assets/textures/effects/` | Texturas para efectos visuales. |
| `assets/audio/music/` | Música. |
| `assets/audio/sfx/` | Efectos de sonido. |
| `assets/fonts/` | Fuentes de la interfaz. |
| `assets/ui/icons/` | Iconos, incluidos los de recompensas e indicadores. |
| `assets/ui/backgrounds/` | Imágenes de fondo de la interfaz. |
| `config/` | Parámetros generales y asociaciones entre franquicias, naves, personajes, minijuegos y recursos. |

Los directorios de modelos de planetas, naves y personajes usan los mismos identificadores: `minecraft`, `southpark`, `smilingfriends`, `halloween`, `regularshow` y `starwars`. `regularshow` corresponde a Un Show Más (Regular Show); su nave y sus personajes de abducción están pendientes de definir, sin decisiones sobre enemigos ni modelos.

## Módulos del juego

| Módulo | Responsabilidad y límites |
| --- | --- |
| `src/core/` | Inicializar el juego, coordinar el renderizado general, administrar escenas, entrada y estado global. No contiene reglas particulares de los minijuegos. |
| `src/scenes/` | Componer las escenas de menú, selección, abducción, sistema solar, estación espacial, minijuegos y final; coordinar sus transiciones usando los demás módulos. |
| `src/solarSystem/` | Crear el sol y los planetas e integrar la ubicación de la estación; gestionar órbitas, rotación y proximidad. Comunicar la posibilidad de visitar un planeta o interactuar con la estación sin implementar sus reglas. |
| `src/spaceStation/` | Gestionar la estación, la interacción y entrada al hangar; consultar las naves desbloqueadas y coordinar la selección voluntaria con `src/ships/`. |
| `src/ships/` | Gestionar las naves disponibles, su bloqueo o desbloqueo y la nave activa; validar selecciones y gestionar pilotaje, movimiento y cámara. No entrega tokens ni decide qué misión se completó. |
| `src/characters/` | Administrar los personajes por franquicia y cargar sus modelos; reutilizar la abducción inicial y por nuevo desbloqueo, y la devolución final. Los modelos permanecen en `assets/`. |
| `src/graphics/shaders/` | Futuros programas de sombreado, una vez elegida la tecnología. |
| `src/graphics/lighting/` | Iluminación del espacio, planetas, plataforma de selección y escenas. |
| `src/graphics/materials/` | Apariencia y propiedades visuales de las superficies. |
| `src/graphics/effects/` | Representación visual del haz de abducción, agujero negro y otros efectos. |
| `src/minigames/common/` | Componentes reutilizables de raycasting, controles y colisiones. No impone raycasting a todos los minijuegos. |
| `src/minigames/minecraft/` | Exploración y búsqueda del diamante. |
| `src/minigames/southpark/` | Recorrido por South Park y puzzle relacionado con Kenny. |
| `src/minigames/smilingfriends/` | Elección de chistes para hacer reír a los personajes. |
| `src/minigames/halloween/` | Escape de personajes de terror. |
| `src/minigames/regularshow/` | Shooter en primera persona con raycasting, ambientado en Un Show Más: disparar a enemigos y acumular puntos. |
| `src/minigames/starwars/` | Minijuego estilo Galaga, sin raycasting. |
| `src/gameplay/` | Registrar misiones completadas y tokens únicos, coordinar desbloqueos con `src/ships/` sin duplicar recompensas y habilitar el final con seis tokens. Gestionar victoria, derrota y reintentos. |
| `src/ui/` | Presentar menús, botones, indicadores y HUD; mostrar las seis naves en la estación y sus estados de bloqueo y desbloqueo. Comunicar acciones sin administrar progreso ni nave activa. |

`docs/` conserva las decisiones de arquitectura, responsables y planificación. `tests/` queda reservado para futuras pruebas de los módulos e integración cuando exista implementación.

## Asociación entre naves y personajes

La configuración asociará el identificador de cada nave con su franquicia, los personajes correspondientes y las referencias a sus modelos:

| Identificador de nave/franquicia | Personajes | Carpeta de modelos |
| --- | --- | --- |
| `minecraft` | Steve | `assets/models/characters/minecraft/` |
| `southpark` | Stan, Kyle, Kenny y Cartman | `assets/models/characters/southpark/` |
| `smilingfriends` | Pim y Charlie | `assets/models/characters/smilingfriends/` |
| `halloween` | Ghost Face | `assets/models/characters/halloween/` |
| `regularshow` | Pendientes de definir | `assets/models/characters/regularshow/` |
| `starwars` | Baby Yoda | `assets/models/characters/starwars/` |

Al seleccionar la nave inicial o desbloquear una nueva nave, la escena consultará esa asociación en `config/` y entregará al módulo `src/characters/` las referencias de los personajes. El mismo módulo cargará uno o varios personajes y ejecutará la secuencia común de abducción con los parámetros necesarios de posición, escala y animación. El estado del juego conservará las asociaciones y los orígenes necesarios para coordinar la devolución final, independientemente de los cambios de nave activa.

Las diferencias entre franquicias se expresarán como datos configurables. No habrá una copia del controlador de personajes ni de las animaciones compartidas por cada franquicia. El formato concreto de estos datos se acordará después de elegir la tecnología; esta entrega no crea archivos de configuración funcionales.

## Progreso, desbloqueos y estación espacial

La estación y estas mecánicas son una arquitectura conceptual planificada, todavía sin implementación. El estado global coordinado por `src/core/` distinguirá los siguientes conceptos, con un único módulo responsable de sus reglas:

| Concepto | Responsable | Regla |
| --- | --- | --- |
| Planetas o misiones completados y tokens | `src/gameplay/` | La primera victoria registra la misión y entrega un único token del planeta. Seis tokens permiten activar el final con cualquier nave activa. |
| Naves desbloqueadas | `src/ships/` | La nave inicial está disponible desde el inicio; las otras cinco se desbloquean por progreso, a solicitud de gameplay. |
| Nave activa | `src/ships/` | Es la nave usada para explorar; después de la selección inicial solo cambia por selección voluntaria en la estación. |

Tener una nave desbloqueada no significa haber completado su planeta: la nave inicial no entrega un token. Completar un planeta no cambia la nave activa.

### Comunicación prevista

1. **Inicio:** la selección permite elegir libremente una de las seis naves. `src/ships/` registra solo esa nave como desbloqueada y activa; los planetas comienzan sin completar y sin tokens. La escena coordina su abducción con `src/characters/` y después entra al sistema solar.
2. **Visita:** la escena conserva la nave activa al entrar al minijuego. Cada minijuego informa únicamente su identificador y resultado a `src/gameplay/`; no modifica naves ni tokens directamente.
3. **Primera victoria:** gameplay registra la misión y su token una sola vez y solicita a ships desbloquear la nave asociada mediante `config/`. Ships informa si pasó de bloqueada a desbloqueada. Solo ese cambio provoca que la escena solicite la abducción a characters antes del regreso. No se activa ni equipa la nave desbloqueada.
4. **Regreso y repetición:** la escena vuelve al sistema solar con la nave usada antes de entrar al minijuego. Una derrota no entrega recompensas; una victoria repetida no entrega otro token, no desbloquea de nuevo y no provoca abducción. La primera victoria del planeta de la nave inicial entrega su token, pero no desbloquea ni repite la abducción porque esa nave ya estaba disponible.
5. **Estación:** la proximidad permite interactuar y la escena abre el hangar. `src/spaceStation/` consulta a ships las seis naves y sus estados y solicita a UI mostrarlos. UI comunica la elección del jugador; la estación la remite a ships, que valida que la nave esté desbloqueada y actualiza la nave activa. Las bloqueadas no se pueden seleccionar. Cambiar entre desbloqueadas es libre y no dispara abducciones. Al salir, la escena retoma la exploración con la selección actual.

Por ejemplo, comenzar con South Park y completar Halloween entrega el token de Halloween, desbloquea su nave y abduce a Ghost Face. El regreso mantiene South Park; únicamente al elegir Halloween en la estación cambia la nave activa. Las otras cuatro naves continúan bloqueadas.

Gameplay centraliza las recompensas; ships conserva el estado de las naves; characters ejecuta animaciones comunes; la estación coordina la interacción; UI presenta opciones y escenas ordena las transiciones. No se crearán copias de estas reglas por franquicia ni subcarpetas adicionales en ships, characters, gameplay o UI. Los contratos y mecanismos concretos de comunicación quedan pendientes de la elección tecnológica.

## Reglas de separación e integración

- Guardar recursos externos en `assets/`, lógica en `src/` y parámetros en `config/`.
- Mantener la iluminación, materiales, shaders y representación de efectos en `src/graphics/`. El movimiento corresponde a naves y sistema solar; las condiciones para activar efectos corresponden a escenas o gameplay.
- Mantener cada minijuego independiente de los otros. Extraer a `src/minigames/common/` únicamente comportamientos reutilizables y coordinar cambios compartidos con el equipo.
- Hacer que cada minijuego comunique su resultado a `src/gameplay/`. Centralizar allí la entrega de tokens para evitar recompensas duplicadas al repetir una misión.
- Mostrar el progreso mediante `src/ui/`, sin duplicar allí las reglas de recompensas, victoria o derrota.
- Coordinar desde escenas el inicio de una visita y el regreso al sistema solar tras una derrota o al terminar una misión.
- Acordar antes de integrar qué información intercambian los módulos: nave activa, naves desbloqueadas, identificador de planeta, resultado de misión, nuevo desbloqueo y progreso. No usar el cambio de nave como efecto secundario de una recompensa. Los contratos concretos quedan pendientes de la elección tecnológica.
- Documentar los cambios que afecten módulos compartidos y evitar dependencias circulares entre ellos.
