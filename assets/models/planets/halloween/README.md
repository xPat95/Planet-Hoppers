# Planeta Halloween

Archivo principal: **[halloween_planet.blend](halloween_planet.blend)** (Blender 5.2). Diseño y decisiones: [guía de Cami](../../../../docs/cami-guidelines.md), sección 2.

La Tierra después del apocalipsis con forma de calavera anatómicamente correcta, gritando: mandíbula abierta sobre su articulación, dientes de adulto, roca coral en estratos como el Gran Cañón, cráteres y pocas grietas de lava. **Es hueca:** las órbitas, la nariz y el foramen magnum atraviesan una cáscara de ~4.5 unidades y adentro hay roca negra verdosa con venas de verde cálido a naranja, con espacio para poner cosas. **Versión v01: el planeta vacío**, sin zonas todavía.

## Organización

Todo depende de `Halloween_Planet_ROOT`, en el origen (propiedad `radius`). Colecciones: `Planet_Base` (la superficie, `HW_Planet_Surface`) y `Presentation` (cámara y luces; excluir al exportar).

- **Tamaño:** factor 22 (igual que South Park); con proporciones de cráneo real mide ~42 de largo, ~31 de ancho y ~46 de alto con la mandíbula abierta. La cara mira hacia -Y y la coronilla hacia +Z.
- **Sin contorno de caricatura:** caras planas low-poly.
- **Luz `InnerGlow`** (colección `Presentation`): luz verde cálida en el centro para ver el interior en Blender.
- **Materiales:** colores planos con prefijo `HW_`, sin texturas. La lava de afuera (`HW_Lava`, `HW_LavaHot`) y el brillo verde de adentro (`HW_InnerGlow`, `HW_InnerGlowHot`) son emisivos: en el motor será el shader animado.

## Presupuesto

| Medida | Valor |
| --- | ---: |
| Triángulos (cáscara de afuera y de adentro) | ~50 000 |

## Edición

**No editar el `.blend` a mano:** se regenera completo con [build_halloween_planet.py](../../../../tools/blender/halloween/build_halloween_planet.py). Ver [instrucciones del script](../../../../tools/blender/halloween/README.md).
