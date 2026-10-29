# Planet Hoppers

Proyecto universitario de Gráficas por Computadora: un videojuego 3D de exploración espacial en un sistema solar ficticio con seis planetas inspirados en Minecraft, South Park, Smiling Friends, Halloween, Un Show Más (Regular Show) y Star Wars.

La estación espacial, el desbloqueo de naves y las demás mecánicas descritas están **planificadas, pero todavía no implementadas**.

## Objetivo general

Desarrollar una experiencia que integre modelado 3D, navegación, órbitas, rotación, iluminación, animaciones y minijuegos en una arquitectura modular. El jugador explorará los seis planetas y completará sus misiones para reunir seis tokens y desbloquear el final.

## Flujo del juego

1. **Menú principal:** introducción espacial con el título Planet Hoppers.
2. **Selección inicial:** elección libre de una única nave entre las seis temáticas, sobre una plataforma iluminada y giratoria. Esa nave queda desbloqueada y activa; las otras cinco permanecen bloqueadas.
3. **Animación inicial:** la nave elegida abduce mediante un haz de luz a los personajes de su franquicia.
4. **Sistema solar:** aparición en un entorno 3D con un sol central, seis planetas con órbitas y rotación propia y una estación espacial interactiva.
5. **Exploración:** pilotaje libre para acercarse a cada planeta o entrar al hangar de la estación espacial. Allí se puede cambiar voluntariamente entre naves desbloqueadas.
6. **Misiones:** un botón de visita aparece por proximidad y permite iniciar el minijuego del planeta. Predominarán el raycasting y la exploración en primera persona; Star Wars será un minijuego estilo Galaga sin raycasting.
7. **Recompensas:** la primera victoria de cada minijuego entrega su token y desbloquea la nave de la franquicia si seguía bloqueada. Solo un nuevo desbloqueo activa su animación de abducción. El regreso al sistema solar conserva la nave usada antes de entrar al minijuego. Perder permite regresar y reintentar sin obtener recompensas.
8. **Final desbloqueable:** reunir los seis tokens habilita terminar el juego independientemente de la nave activa; el sol se transforma en un agujero negro que absorbe los planetas y la nave.
9. **Animación final:** devolución de los personajes abducidos a su lugar de origen.

## Progreso y estación espacial

El progreso distingue tres conceptos: **planetas o misiones completados**, que determinan los tokens obtenidos; **naves desbloqueadas**, disponibles en la estación; y **nave activa**, utilizada para explorar. Elegir una nave inicial no completa su planeta ni entrega su token.

Si se completa por primera vez el planeta de la nave inicial, se obtiene su token, pero no hay otro desbloqueo ni una segunda abducción. Los minijuegos pueden repetirse: las victorias posteriores no duplican tokens, desbloqueos ni animaciones.

Al acercarse a la estación, el jugador puede interactuar para entrar al hangar. Se muestran las seis naves con su estado de bloqueo; solo las desbloqueadas se pueden seleccionar. Al salir, la exploración continúa con la nave seleccionada. Después de la selección inicial, **los cambios de nave solo ocurren por elección voluntaria en la estación espacial**; completar un minijuego nunca cambia automáticamente la nave activa.

Por ejemplo, al empezar con South Park y ganar Halloween, se recibe el token de Halloween, se desbloquea su nave y se reproduce la abducción de Ghost Face. Se regresa con South Park; la estación permite elegir entre South Park y Halloween, mientras las otras cuatro siguen bloqueadas.

## Franquicias y responsables

| Franquicia | Personajes | Minijuego | Responsable |
| --- | --- | --- | --- |
| Minecraft | Steve | Explorar y encontrar un diamante | Pablo |
| South Park | Stan, Kyle, Kenny y Cartman | Recorrer South Park y resolver un puzzle relacionado con Kenny | Pablo |
| Smiling Friends | Pim y Charlie | Hacer reír a personajes eligiendo chistes | Cami |
| Halloween | Ghost Face | Escapar de personajes de terror | Cami |
| Un Show Más (Regular Show) | Mordecai y Rigby | Shooter en primera persona con raycasting: disparar a enemigos y acumular puntos | Norman |
| Star Wars | Baby Yoda | Minijuego estilo Galaga, sin raycasting | Norman |

La nave de Un Show Más es el carrito de golf con alas de cartón; sus personajes son Mordecai y Rigby. Los recursos están modelados; los enemigos y la implementación del minijuego siguen pendientes. Su identificador interno es `regularshow` y seguirá las mismas reglas de tokens, desbloqueo y abducción que las demás franquicias.

Los sistemas generales son trabajo compartido de los tres integrantes. La estación espacial queda como funcionalidad compartida, pendiente de asignación. Ver [distribución de responsabilidades](docs/responsibilities.md).

## Estructura del repositorio

```text
Planet-Hoppers/
├── assets/
│   ├── models/
│   │   ├── planets/
│   │   ├── ships/
│   │   ├── characters/
│   │   └── environment/
│   │       └── spaceStation/
│   ├── textures/
│   │   ├── planets/
│   │   ├── ships/
│   │   ├── characters/
│   │   ├── minigames/
│   │   ├── skybox/
│   │   └── effects/
│   ├── audio/
│   │   ├── music/
│   │   └── sfx/
│   ├── fonts/
│   └── ui/
│       ├── icons/
│       └── backgrounds/
├── src/
│   ├── core/
│   ├── scenes/
│   ├── solarSystem/
│   ├── spaceStation/
│   ├── ships/
│   ├── characters/
│   ├── graphics/
│   │   ├── shaders/
│   │   ├── lighting/
│   │   ├── materials/
│   │   └── effects/
│   ├── minigames/
│   │   ├── common/
│   │   ├── minecraft/
│   │   ├── southpark/
│   │   ├── smilingfriends/
│   │   ├── halloween/
│   │   ├── regularshow/
│   │   └── starwars/
│   ├── gameplay/
│   └── ui/
├── config/
├── docs/
│   ├── architecture.md
│   ├── responsibilities.md
│   └── development-plan.md
├── tests/
├── README.md
└── .gitignore
```

Cada una de las carpetas `assets/models/planets/`, `assets/models/ships/` y `assets/models/characters/` contiene seis subcarpetas: `minecraft/`, `southpark/`, `smilingfriends/`, `halloween/`, `regularshow/` y `starwars/`. Las carpetas vacías se conservan en Git mediante `.gitkeep`.

| Carpeta | Responsabilidad |
| --- | --- |
| `assets/` | Modelos, texturas, audio, fuentes e imágenes de interfaz. Los modelos de personajes se organizan por franquicia en `assets/models/characters/`. |
| `src/core/` | Inicialización, renderizado general, administración de escenas, entrada y estado global. |
| `src/scenes/` | Composición de menú, selección de naves, sistema solar, estación espacial, minijuegos y final, con sus transiciones. |
| `src/solarSystem/` | Sol, planetas, órbitas, rotación e interacción por proximidad. |
| `src/spaceStation/` | Interacción con la estación, entrada al hangar y coordinación de selección y cambio entre naves desbloqueadas. |
| `assets/models/environment/spaceStation/` | Modelos 3D y recursos geométricos de la estación espacial. |
| `src/ships/` | Naves disponibles, bloqueo y desbloqueo, nave activa, movimiento, controles y cámara. |
| `src/characters/` | Carga y manejo compartido de personajes; abducción inicial y por nuevo desbloqueo, y devolución final, sin modelos ni lógica duplicada por franquicia. |
| `src/graphics/` | Shaders, iluminación, materiales y efectos visuales, separados del movimiento y gameplay. |
| `src/minigames/common/` | Raycasting, controles y colisiones reutilizables. |
| `src/minigames/[franquicia]/` | Reglas y lógica exclusivas de cada minijuego. |
| `src/gameplay/` | Misiones completadas, tokens y recompensas únicas; coordinación de desbloqueos, progreso, victoria y derrota. |
| `src/ui/` | Botones, menús, indicadores, HUD e interfaces de estación con estados de bloqueo y desbloqueo. |
| `config/` | Parámetros del juego y asociaciones entre franquicias, naves, personajes y minijuegos. |
| `docs/` | Arquitectura, organización del equipo y planificación. |
| `tests/` | Futuras pruebas de los módulos. |

Consultar las reglas de separación y la asociación entre naves y personajes en [arquitectura](docs/architecture.md).

## Orden de implementación

1. Definición de estructura.
2. Creación de planetas.
3. Creación de naves.
4. Implementación del sistema solar.
5. Navegación y órbitas.
6. Personajes.
7. Minijuegos.
8. Menús, selección de naves y estación espacial.
9. Recompensas y progreso, con condiciones de victoria y derrota.
10. Agujero negro y final.

El [plan de desarrollo](docs/development-plan.md) describe las diez etapas y su estado inicial.

## Tecnologías utilizadas

**Pendiente de definir**. El equipo aún no ha elegido lenguaje, motor gráfico, librerías ni framework.

## Instrucciones de ejecución

**Pendiente de implementación**. Esta entrega contiene estructura, documentación y modelos de planetas; todavía no hay un juego ejecutable ni dependencias del juego que instalar.


## Planetas disponibles para integración

Los siguientes recursos son **planetas del sistema solar**. Sus archivos Blender son las fuentes editables; las exportaciones de juego y sus límites están documentados por franquicia.

| Planeta | Fuente editable | Revisión técnica |
| --- | --- | --- |
| Minecraft | [Minecraft](assets/models/planets/minecraft/README.md) | [Revisión](docs/references/minecraft/technical_review_v02.md) |
| South Park | [South Park](assets/models/planets/southpark/README.md) | [Revisión](docs/planets/southpark.md) |
| Smiling Friends | [Smiling Friends](assets/models/planets/smilingfriends/README.md) | [Organización](assets/models/planets/smilingfriends/README.md) |
| Estrella de la Muerte | [Planeta de Star Wars](assets/models/planets/starwars/README.md) | [Revisión](docs/planets/starwars.md) |
| Domo Espacial | [Planeta de Regular Show](assets/models/planets/regularshow/README.md) | [Revisión](docs/planets/regularshow.md) |


## Naves disponibles como recursos

Los siguientes modelos son naves, independientes de los planetas. Los controles, desbloqueos y animaciones de juego siguen pendientes.

| Temática | Nave o variante | Recursos |
| --- | --- | --- |
| `regularshow` | Carrito de Regular Show | [Fuente y exportaciones](assets/models/ships/regularshow/golfcart/README.md) |
| `starwars` | Caza Naboo N-1 | [Fuente y exportaciones](assets/models/ships/starwars/naboo/README.md) |
