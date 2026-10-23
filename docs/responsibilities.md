# Responsabilidades del equipo

## Minijuegos

| Responsable | Franquicia | Personajes | Alcance | Carpeta de lógica |
| --- | --- | --- | --- | --- |
| Pablo | Minecraft | Steve | Explorar y encontrar un diamante | `src/minigames/minecraft/` |
| Pablo | South Park | Stan, Kyle, Kenny y Cartman | Recorrer South Park y resolver un puzzle relacionado con Kenny | `src/minigames/southpark/` |
| Cami | Smiling Friends | Pim y Charlie | Hacer reír a personajes eligiendo chistes | `src/minigames/smilingfriends/` |
| Cami | Halloween | Ghost Face | Escapar de personajes de terror | `src/minigames/halloween/` |
| Norman | Halo | Master Chief | Disparar a enemigos y acumular puntos | `src/minigames/halo/` |
| Norman | Star Wars | Baby Yoda | Minijuego estilo Galaga, sin raycasting | `src/minigames/starwars/` |

Cada integrante desarrollará las reglas, escenarios y condiciones de resultado de sus dos minijuegos. Sus recursos se ubicarán en las carpetas correspondientes de `assets/`, y cada minijuego comunicará su resultado al sistema compartido de progreso.

## Sistemas generales: trabajo compartido

Pablo, Cami y Norman compartirán la definición e integración de:

- Estructura, documentación y futura elección de tecnologías.
- Modelos de planetas y naves e integración de recursos.
- Inicialización, renderizado general, estado, entrada y administración de escenas (`src/core/`).
- Escenas y transiciones (`src/scenes/`).
- Sol, planetas, órbitas, rotación y proximidad (`src/solarSystem/`).
- Selección, cambio, pilotaje y cámara de naves (`src/ships/`).
- Carga de personajes y animaciones comunes de abducción y devolución (`src/characters/`).
- Shaders, iluminación, materiales y efectos (`src/graphics/`).
- Raycasting, controles y colisiones reutilizables (`src/minigames/common/`).
- Misiones, tokens, reintentos, victoria y derrota (`src/gameplay/`).
- Menús, botones, indicadores y HUD (`src/ui/`).
- Configuraciones, agujero negro, secuencia final y futuras pruebas de integración.

## Coordinación

1. Trabajar en ramas por tarea y mantener los cambios centrados en el módulo correspondiente.
2. Acordar quién modifica cada sistema compartido antes de trabajar simultáneamente sobre los mismos archivos.
3. Mantener los identificadores de franquicia establecidos en la [arquitectura](architecture.md).
4. Coordinar la extracción de lógica común sin copiarla entre minijuegos ni introducir dependencias entre franquicias.
5. Revisar en equipo los cambios que afecten la comunicación entre módulos y actualizar la documentación y el [plan de desarrollo](development-plan.md).

Esta entrega solo prepara la organización; no asigna en exclusiva los sistemas generales ni define todavía herramientas de desarrollo.
