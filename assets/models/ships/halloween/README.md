# Nave Halloween: bruja en escoba

Archivo principal: **[witch_broom.blend](witch_broom.blend)** (Blender 5.2). Diseño: [guía de Cami](../../../../docs/cami-guidelines.md), sección 2.6.

Una de las brujas del planeta sentada de costado en una escoba (como amazona, con las dos piernas juntas colgando de un lado), con el torso hacia adelante e inclinada como volando rápido, con ojos de cuenta rojos y brillantes (con un brillito blanco), chicos y separados, dos círculos rosados en los cachetes, sin nariz ni boca, con un cuello que se nota y el pelo de siempre de las brujas pero azul azabache (muy oscuro, se distingue del vestido negro) y con el borde de abajo en ondas redondas, con una mano en el palo y la otra saludando con el codo doblado; las manos son un poco más grandes y tienen pulgar. Lleva una falda de tela asimétrica, con pliegues y el borde ondulado: corta adelante, apoyada sobre el regazo hasta la mitad del muslo (las rodillas y las medias se ven), y larga atrás, donde el viento la lleva hacia atrás, con pliegues hondos, el borde ondulado y redondeado (sin puntas) y un vaivén de tela (nace de la cintura, sin escalón); adelante y detrás del asiento la tela cae sobre el palo, sin atravesarlo, y el final de atrás se enrosca hacia arriba y deja ver un poco el palo. Detrás de la paja deja una estela corta y ondulada de estrellas de colores de Halloween (verde, morado, naranja, rojo y lima).

## Contenido

| Objeto | Qué tiene |
| --- | --- |
| `HW_Witch_Ride` | La bruja, de [witch.blend](../../characters/halloween/witch.blend) (pose `ride` de `build_witch.py`), con el cuerpo de una sola pieza (`smooth=True`): torso, cuello, cabeza, brazos, manos y piernas unidos en suave, sin costuras en hombros, codos, cadera ni cuello, y con los bordes de color derechos (sisas, escote, borde del vestido en el muslo y rayas de las medias). Las rodillas van juntas, pegadas al palo. Origen en el asiento. |
| `HW_Broom` | La escoba: palo de madera con la punta redondeada, que pasa justo bajo la cola (apoyada encima, sin atravesarla) y por la mano, girado 14° bajo ella (`BROOM_YAW` en `build_witch.py`: ella mira para adelante y, de atrás, el palo y la paja asoman al costado; el brazo que lo agarra se acomoda solo para que la mano quede sobre el palo); atado morado (como la cinta del sombrero) y paja tupida: 80 mechones de dos tonos en tres capas que se abren hacia atrás. |
| `HW_StarTrail` | Estela: 16 estrellas infladas (gorditas, con puntas cortas, anchas y redondas, abombadas en el medio, con lustre) en colores de Halloween (verde veneno, morado oscuro, naranja calabaza, rojo sangre y verde lima), cada vez más chicas, y 30 chispitas sueltas alrededor. |

Todo depende de `Halloween_Ship_ROOT`, en el origen. La bruja mira hacia -Y (en vuelo se gira 180° con `config/ships.json`).

## Tamaño en el juego

El juego escala cada nave para que su lado más largo mida `length` (`config/ships.json`). Acá el lado más largo es de la punta del palo al final de la estela, así que va con `length` 7.0 para que la bruja se vea del tamaño de las otras naves. En el hangar, `display_scale` de `src/graphics/hangar.rs` la agranda 1.5 veces, por lo mismo.

## Presupuesto

~81 000 triángulos (unas 17 000 son de las estrellas infladas), casi todos del cuerpo de la bruja (GLB de ~0.7 MB).

Capturas: `docs/references/halloween/broom_*.png` (tres cuartos, costado, frente, atrás y de cerca).

## Edición

**No editar el `.blend` a mano:** se regenera con [build_witch_broom.py](../../../../tools/blender/halloween/build_witch_broom.py), después de [build_witch.py](../../../../tools/blender/halloween/build_witch.py) (la pose de la bruja está en `POSES["ride"]`).

## Exportación para el juego

[witch_broom.glb](exports/witch_broom.glb), generado con [export_solar_models.py](../../../../tools/blender/solarsystem/export_solar_models.py) (`-- halloween_ship`), sin la colección `Presentation`. Volver a exportar después de cambiar el modelo.
