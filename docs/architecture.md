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
| `config/` | Parámetros generales, propiedades del sistema solar y naves, y asociaciones de naves con personajes y recursos. |

Los directorios de modelos de planetas, naves y personajes usan los mismos identificadores: `minecraft`, `southpark`, `smilingfriends`, `halloween`, `halo` y `starwars`.

## Módulos del juego

| Módulo | Responsabilidad y límites |
| --- | --- |
| `src/core/` | Inicializar el juego, coordinar el renderizado general, administrar escenas, entrada y estado global. No contiene reglas particulares de los minijuegos. |
| `src/scenes/` | Componer las escenas de menú, selección, abducción, sistema solar, minijuegos y final; coordinar sus transiciones usando los demás módulos. |
| `src/solarSystem/` | Crear el sol y los planetas; gestionar órbitas, rotación y proximidad. Comunicar la posibilidad de visitar un planeta sin implementar su minijuego. |
| `src/ships/` | Definir y seleccionar naves; gestionar pilotaje, movimiento y cámara. Comunicar la nave elegida para resolver sus personajes mediante configuración. |
| `src/characters/` | Cargar y manejar personajes; ejecutar la lógica común de abducción y devolución. Los modelos permanecen en `assets/`. |
| `src/graphics/shaders/` | Futuros programas de sombreado, una vez elegida la tecnología. |
| `src/graphics/lighting/` | Iluminación del espacio, planetas, plataforma de selección y escenas. |
| `src/graphics/materials/` | Apariencia y propiedades visuales de las superficies. |
| `src/graphics/effects/` | Representación visual del haz de abducción, agujero negro y otros efectos. |
| `src/minigames/common/` | Componentes reutilizables de raycasting, controles y colisiones. No impone raycasting a todos los minijuegos. |
| `src/minigames/minecraft/` | Exploración y búsqueda del diamante. |
| `src/minigames/southpark/` | Recorrido por South Park y puzzle relacionado con Kenny. |
| `src/minigames/smilingfriends/` | Elección de chistes para hacer reír a los personajes. |
| `src/minigames/halloween/` | Escape de personajes de terror. |
| `src/minigames/halo/` | Disparos a enemigos y acumulación de puntos. |
| `src/minigames/starwars/` | Minijuego estilo Galaga, sin raycasting. |
| `src/gameplay/` | Administrar misiones, resultados, progreso y un token por planeta; habilitar el final al reunir los seis. Gestionar victoria, derrota y reintentos. |
| `src/ui/` | Presentar menús, botones de visita, indicadores y HUD; comunicar acciones a los módulos responsables. |

`docs/` conserva las decisiones de arquitectura, responsables y planificación. `tests/` queda reservado para futuras pruebas de los módulos e integración cuando exista implementación.

## Asociación entre naves y personajes

La configuración asociará el identificador de cada nave con su franquicia, los personajes correspondientes y las referencias a sus modelos:

| Identificador de nave/franquicia | Personajes | Carpeta de modelos |
| --- | --- | --- |
| `minecraft` | Steve | `assets/models/characters/minecraft/` |
| `southpark` | Stan, Kyle, Kenny y Cartman | `assets/models/characters/southpark/` |
| `smilingfriends` | Pim y Charlie | `assets/models/characters/smilingfriends/` |
| `halloween` | Ghost Face | `assets/models/characters/halloween/` |
| `halo` | Master Chief | `assets/models/characters/halo/` |
| `starwars` | Baby Yoda | `assets/models/characters/starwars/` |

Al seleccionar una nave, la escena consultará esa asociación en `config/` y entregará al módulo `src/characters/` las referencias de los personajes. El mismo módulo cargará uno o varios personajes y ejecutará la secuencia común de abducción con los parámetros necesarios de posición, escala y animación. La selección y su origen se conservarán en el estado del juego para coordinar la devolución final.

Las diferencias entre franquicias se expresarán como datos configurables. No habrá una copia del controlador de personajes ni de las animaciones compartidas por cada franquicia. El formato concreto de estos datos se acordará después de elegir la tecnología; esta entrega no crea archivos de configuración funcionales.

## Reglas de separación e integración

- Guardar recursos externos en `assets/`, lógica en `src/` y parámetros en `config/`.
- Mantener la iluminación, materiales, shaders y representación de efectos en `src/graphics/`. El movimiento corresponde a naves y sistema solar; las condiciones para activar efectos corresponden a escenas o gameplay.
- Mantener cada minijuego independiente de los otros. Extraer a `src/minigames/common/` únicamente comportamientos reutilizables y coordinar cambios compartidos con el equipo.
- Hacer que cada minijuego comunique su resultado a `src/gameplay/`. Centralizar allí la entrega de tokens para evitar recompensas duplicadas al repetir una misión.
- Mostrar el progreso mediante `src/ui/`, sin duplicar allí las reglas de recompensas, victoria o derrota.
- Coordinar desde escenas el inicio de una visita y el regreso al sistema solar tras una derrota o al terminar una misión.
- Acordar antes de integrar qué información intercambian los módulos: nave seleccionada, identificador de planeta, resultado de misión y progreso. Los contratos concretos quedan pendientes de la elección tecnológica.
- Documentar los cambios que afecten módulos compartidos y evitar dependencias circulares entre ellos.
