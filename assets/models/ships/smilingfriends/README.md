# Nave Smiling Friends

Archivo principal: **[smilingfriends_ship.blend](smilingfriends_ship.blend)** (Blender 5.2). Diseño: [guía de Cami](../../../../docs/cami-guidelines.md), sección 1.7.

El OVNI de T2 EP6 "Charlie, Pim, and Bill vs. the Alien": platillo plateado con cúpula de vidrio transparente, y adentro Pim y Charlie frente a la audiencia: Pim gritando y Charlie saludando.

## Contenido

| Objeto | Qué tiene |
| --- | --- |
| `SF_Ship_Hull` | Platillo de perfil revolucionado (radio 3), con el piso de la cabina en el centro. |
| `SF_Ship_Dome`, `SF_Ship_DomeRing` | Cúpula de vidrio transparente (sin costillas) y el anillo plateado donde apoya. |
| `SF_Ship_Details` | Calcomanías planas sin contorno. Arriba: anillo alrededor de la cúpula, paneles irregulares de líneas finas y cinco placas oscuras. Abajo: centro de ocho gajos, anillos, tres luces ovaladas en su hueco y paneles oscuros de tres celdas entre ellas. |
| `SF_Ship_Beam` | Haz verde translúcido de abducción, aparte para encenderlo o apagarlo en el juego. |
| `SF_Ship_Cabin` | Cabina: dos asientos acolchados con respaldo rectangular de esquinas redondeadas, cojín y hueco en el piso; y `SF_Cabin_Console`, un tablero de mando chico al frente. |
| `SF_Pim`, `SF_Charlie` | "Stickers inflados": la silueta de [pim.png](../../characters/smilingfriends/cutouts/pim.png) y [charlie.png](../../characters/smilingfriends/cutouts/charlie.png) recortada en una malla e inflada como un almohadón (gruesa en el medio, fina en los bordes), con el dibujo adelante y atrás. Parados delante de sus sillas, mirando al frente. Material `*_Sticker`: el juego lo dibuja con los colores planos del dibujo, sin luz. |

Las imágenes proceden de `pimpic.png` y `charliepic.jpg` en la raíz del proyecto (la versión transparente de Charlie se preparó con ImageGen). Si cambian los PNG de `cutouts/`, se vuelve a generar la nave: la malla sale de su silueta.

Todo depende de `SmilingFriends_Ship_ROOT`, en el origen. Los personajes miran hacia -Y (en vuelo se giran 180° con `config/ships.json`). Son objetos comunes: las sillas los tapan solo cuando están delante, y de costado se ve un cuerpo redondeado.

El modificador `Outline` (contorno de caricatura) va en el casco y la cabina. Esta nave lo exporta como geometría y el juego lo dibuja en negro.

## Presupuesto

~105 000 triángulos en el `.blend` (Pim ~45 000 y Charlie ~48 000). El GLB tiene ~95 000.

Capturas: `docs/references/smilingfriends/ship_*.png`. `ship_figures`, `ship_figures_side` y `ship_figures_back` muestran a los personajes de frente, de costado y desde atrás.

## Edición

**No editar el `.blend` a mano:** se regenera con [build_smilingfriends_ship.py](../../../../tools/blender/smilingfriends/build_smilingfriends_ship.py). Ver [instrucciones de los scripts](../../../../tools/blender/smilingfriends/README.md).

## Exportación para el juego

[smilingfriends_ship.glb](exports/smilingfriends_ship.glb): GLB para el [sistema solar](../../../../docs/solar-system.md), generado con [export_solar_models.py](../../../../tools/blender/solarsystem/export_solar_models.py) sin modificar el `.blend` (se excluyen `Presentation`, cámaras y luces). Tiene ~95 000 triángulos; lleva las imágenes de los personajes adentro. Volver a exportar después de cambiar el modelo.

El haz de abducción queda en el `.blend` como efecto aparte, pero se excluye del GLB de vuelo y selección para que no agrande la caja de encuadre ni eleve visualmente la nave.
