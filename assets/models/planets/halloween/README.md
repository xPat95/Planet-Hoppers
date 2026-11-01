# Planeta Halloween

Archivo principal: **[halloween_planet.blend](halloween_planet.blend)** (Blender 5.2). Diseño y decisiones: [guía de Cami](../../../../docs/cami-guidelines.md), sección 2.

La Tierra después del apocalipsis con forma de calavera anatómicamente correcta, gritando: mandíbula abierta sobre su articulación, dientes de adulto, roca roja en estratos como el Gran Cañón, cráteres y grietas de lava roja profunda. **Es hueca:** las órbitas, la nariz y el foramen magnum atraviesan una cáscara de ~4.5 unidades y adentro hay roca negra verdosa con venas de verde cálido a naranja, con espacio para poner cosas.

## Organización

Todo depende de `Halloween_Planet_ROOT`, en el origen (propiedad `radius`). Colecciones: `Planet_Base` (la superficie, `HW_Planet_Surface`), `Zones` (`HW_Mouth_Snakes`: 18 serpientes finas y escamosas con la cola apoyada en la mandíbula y los dientes: gatean hacia adelante o cuelgan de los dientes), `Zones` > `Devils` (46 diablitos rojos bailando en grupos en todas las grietas, copias de [devil.blend](../../characters/halloween/devil.blend)), `Zones` > `Witches` (23 brujas en 20 copias, sus sombras de contacto y `HW_Music_Notes`, notas que brillan alrededor de las brujas y los diablitos que bailan: copias de los modelos de [witch.blend](../../characters/halloween/witch.blend)) y `Presentation` (cámara y luces; excluir al exportar).

- **Tamaño:** factor 22 (igual que South Park); con proporciones de cráneo real mide ~42 de largo, ~31 de ancho y ~46 de alto con la mandíbula abierta. La cara mira hacia -Y y la coronilla hacia +Z.
- **Sin contorno de caricatura:** caras planas low-poly.
- **Luz `InnerGlow`** (colección `Presentation`): luz verde cálida en el centro para ver el interior en Blender.
- **Materiales:** colores planos con prefijo `HW_`, sin texturas. La lava de afuera (`HW_Lava`, `HW_LavaHot`) y el brillo verde de adentro (`HW_InnerGlow`, `HW_InnerGlowHot`) son emisivos: en el motor será el shader animado.

## Presupuesto

| Medida | Valor |
| --- | ---: |
| Triángulos (cáscara de afuera y de adentro) | ~40 000 |
| Brujas (20 copias) | ~34 000 |
| Diablitos (46 copias) | ~16 800 |
| Serpientes de la boca | ~3 900 |
| Notas musicales | ~4 300 |
| **Total del planeta** | **~98 700** |

## Edición

**No editar el `.blend` a mano:** se regenera completo con [build_halloween_planet.py](../../../../tools/blender/halloween/build_halloween_planet.py). Ver [instrucciones del script](../../../../tools/blender/halloween/README.md).
