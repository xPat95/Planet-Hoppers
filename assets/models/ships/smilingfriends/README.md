# Nave Smiling Friends

Archivo principal: **[smilingfriends_ship.blend](smilingfriends_ship.blend)** (Blender 5.2). Diseño: [guía de Cami](../../../../docs/cami-guidelines.md), sección 1.7.

El OVNI de T2 EP6 "Charlie, Pim, and Bill vs. the Alien": platillo plateado con cúpula negra brillante.

## Contenido

| Objeto | Qué tiene |
| --- | --- |
| `SF_Ship_Hull` | Platillo de perfil revolucionado, con contorno de caricatura. |
| `SF_Ship_Dome` | Cúpula negra brillante. |
| `SF_Ship_Details` | Calcomanías planas sin contorno. Arriba: anillo alrededor de la cúpula, paneles irregulares de líneas finas y cinco placas oscuras. Abajo: centro de ocho gajos, anillos, tres luces ovaladas en su hueco y paneles oscuros de tres celdas entre ellas. |
| `SF_Ship_Beam` | Haz verde translúcido de abducción, aparte para encenderlo o apagarlo en el juego. |

Todo depende de `SmilingFriends_Ship_ROOT`, en el origen. Radio del platillo: 2 unidades. El modificador `Outline` es solo para la vista en Blender; omitirlo al exportar.

## Presupuesto

~5 300 triángulos en total (sin contorno).

## Edición

**No editar el `.blend` a mano:** se regenera con [build_smilingfriends_ship.py](../../../../tools/blender/smilingfriends/build_smilingfriends_ship.py). Ver [instrucciones de los scripts](../../../../tools/blender/smilingfriends/README.md).
