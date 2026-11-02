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
| Cara inferior | Masa de océano y hielo con picos que cuelgan; cráter del infierno con llamas y Satanás (cuerpo esculpido, musculoso) jugando en su PC gamer. |

## Organización

Todo depende de `SmilingFriends_Planet_ROOT`, en el origen. Colecciones: `Planet_Base`, `Land`, `Rim` (nubes), `Hell`, `Zones` (una subcolección por zona y `Roads`), `LOD_Far` (nubes lejanas, ocultas) y `Presentation` (cámara y luces; excluir al exportar).

- **Radio del disco:** 24 unidades (South Park: esfera de radio 22).
- **Contorno de caricatura:** modificador `Outline` (Solidify invertido) en casi todas las mallas, solo para la vista en Blender. La exportación al juego lo conserva como geometría y el juego lo dibuja en negro. El Forest Demon no lleva contorno (estilo plastilina).
- **Materiales:** colores planos con prefijo `SF_`, sin texturas externas. Los letreros son texto convertido a malla.

## Presupuesto

| Medida | Valor |
| --- | ---: |
| Triángulos de cerca (sin contorno) | ~361 000 |
| Triángulos de lejos (nubes `LOD_Far`) | ~248 000 |

Calidad: `Q` (variable `SF_QUALITY`, 1.6 por defecto) multiplica los lados de todas las piezas redondas (árboles, nubes, arbustos, rocas, Shmaloogles...); las piezas facetadas a propósito (pirámides, agujas de 3 o 4 lados) no cambian. `SF_QUALITY=1` da la versión liviana anterior (~98 000). Satanás tiene el cuerpo esculpido (`SF_Hell_Satan_Body`: remesh y suavizado como el Forest Demon, con músculos y cara marcados; sin contorno), con `SATAN_TRIS` caras. Siguiente ahorro recomendado: versión lejana de los árboles.

## Edición

**No editar el `.blend` a mano:** se regenera completo con [build_smilingfriends_planet.py](../../../../tools/blender/smilingfriends/build_smilingfriends_planet.py). Ver [instrucciones de los scripts](../../../../tools/blender/smilingfriends/README.md). El Forest Demon vive en [forest_demon.blend](../../characters/smilingfriends/forest_demon.blend) y el planeta lo importa.

## Exportación para el juego

[smilingfriends_planet.glb](exports/smilingfriends_planet.glb): GLB para el [sistema solar](../../../../docs/solar-system.md), generado con [export_solar_models.py](../../../../tools/blender/solarsystem/export_solar_models.py) sin modificar el `.blend` (se excluyen `Presentation`, cámaras y luces). ~736 000 triángulos: lleva el contorno `Outline` como geometría (material `SF_Outline`, que el juego dibuja en negro), sin `LOD_Far`. Volver a exportar después de cambiar el modelo.

## Mejoras de calidad (noviembre 2026)

- Nubes más redondas (más lados por bola) y todos los árboles iguales: los cuatro de la calle de la ciudad usan el mismo árbol de copas redondas que el resto, y todas las copas tienen más lados.
- Casas Shmaloogle de una sola pieza (perfil girado, sin costura entre pared y cúpula), con puerta de arco y marco pegados a la pared, escalón, y ventana redonda hundida con su cruz.
- Edificios de Río con bandas de ventanas por piso y cornisa en el techo.
- Picos de hielo de la cara inferior: racimos de cristales hexagonales facetados con la punta más clara.
- Cristo Redentor modelado: pedestal escalonado, túnica que se angosta en la faja, brazos abiertos con las mangas colgando, manos, cuello y cabeza; el Corcovado es un cerro redondeado.
