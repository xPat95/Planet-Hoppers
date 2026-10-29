# End Ship de Minecraft

Modelo principal: [end_ship.blend](end_ship.blend). Abrir directamente en Blender; no requiere ejecutar scripts. Archivo verificado con Blender 5.4.0 Alpha.

La geometría del casco, cubierta, cabina, mástil, remate hueco, escaleras y slabs procede de la plantilla original `data/minecraft/structure/end_city/ship.nbt`, extraída de la instalación local de Minecraft Java 1.21.11. Se conservan posiciones y orientaciones de bloques, incluidas las esquinas de escaleras y escaleras invertidas. La plantilla ocupa 13 × 24 × 29 bloques (ancho × alto × largo). Los marcadores técnicos de generación no se modelan.

## Revisión visual

- [Perspectiva de tres cuartos](../../../../docs/references/minecraft/end_ship/end_ship_three_quarter.png).
- [Vista lateral ortográfica](../../../../docs/references/minecraft/end_ship/end_ship_side.png).
- [Vista superior](../../../../docs/references/minecraft/end_ship/end_ship_top.png).
- [Informe de validación](../../../../docs/references/minecraft/end_ship/validation.json).

## Organización e integración

La escena `Minecraft_End_Ship` contiene nueve mallas y 8.458 caras cuadrangulares. Las caras de contacto ocultas entre bloques y entre componentes de las escaleras se eliminan mediante una cuadrícula de medio bloque. Las caras expuestas conservan subdivisiones de medio bloque; no se aplica una reducción destructiva del detalle.

Todos los componentes del modelo dependen de `EndShip_ROOT`. Las colecciones separan `Hull`, `Bow`, `Deck`, `Mast`, `Cabin`, `Crown`, `End_Rods`, `Dragon_Head` e `Interior`. `EndShip_Presentation` contiene únicamente cámara y luces, fuera de la jerarquía de exportación.

- Escala: 1 unidad de Blender = 1 bloque, escena en metros.
- Frente: **+Y**, hacia la Dragon Head. Arriba: **+Z**.
- Origen: eje central de la nave a la altura de la cubierta principal; corresponde al punto Minecraft `(6.5, 8, 14.5)` de la plantilla.
- Seleccionar la jerarquía `EndShip_ROOT` para una futura exportación, excluyendo la presentación.
- Materiales independientes con prefijo `EndShip_`, caras planas, sin bevels ni modificadores.
- No incluye físicas, controles, movimiento ni lógica del videojuego. El formato de exportación se definirá al elegir el motor.

## Texturas

Las siete imágenes proporcionadas en `assets/textures/ships/` se utilizan sin modificar sus archivos originales:

| Imagen | Uso |
| --- | --- |
| `purpur_block.png` | Casco, cubierta, cabina, escaleras y slabs. |
| `purpur_pillar_side.png` | Laterales del mástil y pilares; vetas orientadas por eje. |
| `purpur_pillar_top.png` | Extremos de los pilares. |
| `end_stone_bricks.png` | Paredes de la cabina. |
| `purple_stained_glass.png` | Ventanas, alfa 0,38 y transmisión 0,18. Sustituye el cristal magenta de la plantilla original por el recurso solicitado. |
| `end_rod.png` | Seis End Rods, con geometría del modelo original y UV del atlas. |
| `obsidian.png` | Bloques interiores originales y material provisional de la cabeza de dragón. |

Se añaden cuatro texturas originales de la misma instalación local en `assets/textures/ships/minecraft_end_ship/`: `ladder.png`, `chest.png` (atlas `entity/chest/normal.png`), `brewing_stand.png` y `brewing_stand_base.png`. Permiten conservar la escalera de acceso al mástil y los accesorios interiores sin sustituirlos por materiales inventados. Son recursos de Minecraft/Mojang; su inclusión no otorga derechos adicionales de distribución.

Las once imágenes están **empaquetadas en el .blend**, con rutas externas relativas como referencia. Los materiales usan interpolación `Closest` y repetición de un bloque por unidad. El archivo puede trasladarse sin perder texturas.

## Simplificaciones explícitas

La Dragon Head usa una silueta cúbica de cráneo, hocico, mandíbula y cuernos, con obsidiana provisional y ojos morados. No reproduce el atlas original de la entidad. Los dos cofres interiores tienen UV simplificadas y el soporte de pociones conserva base y columna; no se incluyen botellas, inventarios, Elytra, marcos de objetos, shulkers ni animaciones. Estas limitaciones afectan a los accesorios, no a la distribución de bloques de la estructura del barco.

Los [scripts](../../../../tools/blender/minecraft/end_ship/README.md) y los [datos de procedencia](../../../../docs/references/minecraft/end_ship/source_provenance.json) documentan la generación. No se modificaron los archivos de otros planetas o naves.
