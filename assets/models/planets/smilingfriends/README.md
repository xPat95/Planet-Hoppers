# Planeta Smiling Friends

Archivo principal: **[smilingfriends_planet.blend](smilingfriends_planet.blend)** (Blender 5.2). Diseño y decisiones: [guía de Cami](../../../../docs/cami-guidelines.md), sección 1.

La Tierra plana de la serie: un disco con océano, una masa de tierra central y dos islas, borde de nubes y, por debajo, una montaña de hielo invertida con el infierno en el centro. La distribución sigue el [boceto de Cami](../../../../docs/references/smilingfriends/smilingfriends_layout_sketch.png).

## Contenido

| Zona | Qué tiene |
| --- | --- |
| Calle central | Esfera de Smiling Friends Inc. frente a la de Frowning Friends, calle con edificios (RON'S DELI, ASYLUM, TOILET EMPORIUM, ANTIQUE STORE) y siluetas moradas de la ciudad al fondo. |
| Bosque de otoño | Detrás de Frowning Friends, degradado de verde a amarillo, naranja y rojo. |
| Spamtopia | Degradado pasto → desierto (letrero SPAMTOPIA) → verde neón, carretera arcoíris, faroles y muchos edificios psicodélicos. |
| Casa de Mr. Frog | Península con camino en zigzag por cinco paisajes en bandas (rocas, bosque, hielo, desierto, pantano) hasta su casa con techo de escamas, estanque y luciérnagas. |
| Shmaloogleville | Carretera sobre dos colinas, pueblo amurallado con arco, casas de gomita, tallos en espiral y Shmaloogles; castillo del mago de agujas oscuras. |
| Enchanted Forest | Parque cercado unido a la tierra central: árboles de troncos de colores, hongos, arbustos y castillo amarillo de Mip. |
| Bosque del Forest Demon | Isla de árboles fríos y grisáceos con el Forest Demon (modelo propio importado) y puente de madera arqueado. |
| Brasil | Isla con forma de Brasil, Corcovado con el Cristo Redentor, edificios de Río y selva. |
| Cara inferior | Masa de océano y hielo con picos que cuelgan; cráter del infierno con llamas y Satanás jugando en su PC gamer. |

## Organización

Todo depende de `SmilingFriends_Planet_ROOT`, en el origen. Colecciones: `Planet_Base`, `Land`, `Rim` (nubes), `Hell`, `Zones` (una subcolección por zona y `Roads`), `LOD_Far` (nubes lejanas, ocultas) y `Presentation` (cámara y luces; excluir al exportar).

- **Radio del disco:** 24 unidades (South Park: esfera de radio 22).
- **Contorno de caricatura:** modificador `Outline` (Solidify invertido) en casi todas las mallas, solo para la vista en Blender. No es geometría del modelo: omitirlo al exportar; en el motor se hará como shader de contorno. El Forest Demon no lleva contorno (estilo plastilina).
- **Materiales:** colores planos con prefijo `SF_`, sin texturas externas. Los letreros son texto convertido a malla.

## Presupuesto

| Medida | Valor |
| --- | ---: |
| Triángulos de cerca (sin contorno) | ~98 200 |
| Triángulos de lejos (nubes `LOD_Far`) | ~82 000 |

Las nubes lejanas son el mismo conjunto con menos segmentos: el motor las cambia cuando el planeta está lejos. Siguiente ahorro recomendado: versión lejana de los árboles.

## Edición

**No editar el `.blend` a mano:** se regenera completo con [build_smilingfriends_planet.py](../../../../tools/blender/smilingfriends/build_smilingfriends_planet.py). Ver [instrucciones de los scripts](../../../../tools/blender/smilingfriends/README.md). El Forest Demon vive en [forest_demon.blend](../../characters/smilingfriends/forest_demon.blend) y el planeta lo importa.
