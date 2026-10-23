# Planet Hoppers

Proyecto universitario de Gráficas por Computadora: un videojuego 3D de exploración espacial en un sistema solar ficticio con seis planetas inspirados en Minecraft, South Park, Smiling Friends, Halloween, Halo y Star Wars.

## Objetivo general

Desarrollar una experiencia que integre modelado 3D, navegación, órbitas, rotación, iluminación, animaciones y minijuegos en una arquitectura modular. El jugador explorará los seis planetas y completará sus misiones para reunir seis tokens y desbloquear el final.

## Flujo del juego

1. **Menú principal:** introducción espacial con el título Planet Hoppers.
2. **Selección de nave:** elección entre seis naves temáticas, una por franquicia, sobre una plataforma iluminada y giratoria.
3. **Animación inicial:** la nave elegida abduce mediante un haz de luz a los personajes de su franquicia.
4. **Sistema solar:** aparición en un entorno 3D con un sol central y seis planetas con órbitas y rotación propia.
5. **Exploración:** pilotaje libre para acercarse a cada planeta.
6. **Misiones:** un botón de visita aparece por proximidad y permite iniciar el minijuego del planeta. Predominarán el raycasting y la exploración en primera persona; Star Wars será un minijuego estilo Galaga sin raycasting.
7. **Recompensas:** completar cada minijuego entrega su token. Perder devuelve al jugador al sistema solar para volver a intentarlo.
8. **Final desbloqueable:** reunir los seis tokens habilita terminar el juego; el sol se transforma en un agujero negro que absorbe los planetas y la nave.
9. **Animación final:** devolución de los personajes abducidos a su lugar de origen.

## Franquicias y responsables

| Franquicia | Personajes | Minijuego | Responsable |
| --- | --- | --- | --- |
| Minecraft | Steve | Explorar y encontrar un diamante | Pablo |
| South Park | Stan, Kyle, Kenny y Cartman | Recorrer South Park y resolver un puzzle relacionado con Kenny | Pablo |
| Smiling Friends | Pim y Charlie | Hacer reír a personajes eligiendo chistes | Cami |
| Halloween | Ghost Face | Escapar de personajes de terror | Cami |
| Halo | Master Chief | Disparar a enemigos y acumular puntos | Norman |
| Star Wars | Baby Yoda | Minijuego estilo Galaga, sin raycasting | Norman |

Los sistemas generales son trabajo compartido de los tres integrantes. Ver [distribución de responsabilidades](docs/responsibilities.md).

## Estructura del repositorio

```text
Planet-Hoppers/
├── assets/
│   ├── models/
│   │   ├── planets/
│   │   ├── ships/
│   │   ├── characters/
│   │   └── environment/
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
│   │   ├── halo/
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

Cada una de las carpetas `assets/models/planets/`, `assets/models/ships/` y `assets/models/characters/` contiene seis subcarpetas: `minecraft/`, `southpark/`, `smilingfriends/`, `halloween/`, `halo/` y `starwars/`. Las carpetas vacías se conservan en Git mediante `.gitkeep`.

| Carpeta | Responsabilidad |
| --- | --- |
| `assets/` | Modelos, texturas, audio, fuentes e imágenes de interfaz. Los modelos de personajes se organizan por franquicia en `assets/models/characters/`. |
| `src/core/` | Inicialización, renderizado general, administración de escenas, entrada y estado global. |
| `src/scenes/` | Composición de menú, selección de naves, sistema solar, minijuegos y final. |
| `src/solarSystem/` | Sol, planetas, órbitas, rotación e interacción por proximidad. |
| `src/ships/` | Definición, selección, movimiento, controles y cámara de las naves. |
| `src/characters/` | Carga y manejo compartido de personajes y animaciones de abducción y devolución, sin modelos ni lógica duplicada por franquicia. |
| `src/graphics/` | Shaders, iluminación, materiales y efectos visuales, separados del movimiento y gameplay. |
| `src/minigames/common/` | Raycasting, controles y colisiones reutilizables. |
| `src/minigames/[franquicia]/` | Reglas y lógica exclusivas de cada minijuego. |
| `src/gameplay/` | Misiones, tokens, recompensas, progreso, victoria y derrota. |
| `src/ui/` | Botones, menús, indicadores y HUD. |
| `config/` | Parámetros del juego y asociaciones entre naves y personajes. |
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
8. Menús, selección y cambio de naves.
9. Recompensas, condiciones de victoria y derrota.
10. Agujero negro y final.

El [plan de desarrollo](docs/development-plan.md) describe las diez etapas y su estado inicial.

## Tecnologías utilizadas

**Pendiente de definir**. El equipo aún no ha elegido lenguaje, motor gráfico, librerías ni framework.

## Instrucciones de ejecución

**Pendiente de implementación**. Esta entrega contiene únicamente la estructura de carpetas y documentación; todavía no hay un juego ejecutable ni dependencias que instalar.
