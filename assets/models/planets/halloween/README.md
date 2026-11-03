# Planeta Halloween

Archivo principal: **[halloween_planet.blend](halloween_planet.blend)** (Blender 5.2). Diseño y decisiones: [guía de Cami](../../../../docs/cami-guidelines.md), sección 2.

La Tierra después del apocalipsis con forma de calavera anatómicamente correcta, gritando: mandíbula abierta sobre su articulación, dientes de adulto, roca roja en estratos como el Gran Cañón, cráteres y grietas de lava roja profunda. **Es hueca:** las órbitas, la nariz y el foramen magnum atraviesan una cáscara de ~4.5 unidades y adentro hay un mar de lava (costras oscuras sobre lava roja y naranja, con pocas zonas verdes), con espacio para poner cosas.

## Organización

Todo depende de `Halloween_Planet_ROOT`, en el origen (propiedad `radius`). Colecciones: `Planet_Base` (la superficie, `HW_Planet_Surface`), `LOD_Far` (versión lejana, oculta), `Zones` (`HW_Chin_Blood`: un solo líquido rojo brillante que sale solo por los dientes de abajo, hecho como forma de distancia y unido en suave: charco grueso sobre las coronas, una capa por el frente que termina en arcos de media luna de alturas distintas, gotas que bajan como regueros chatos desde donde se juntan los arcos, gotas pesadas que cuelgan al borde del mentón; dos terminan en gotas y tres se curvan en una J donde se sientan brujas felices), `Zones` > `Devils` (38 diablitos rojos bailando, más grandes que antes, repartidos parejo por todas las grietas, copias de [devil.blend](../../characters/halloween/devil.blend)), `Zones` > `Witches` (10 brujas en 9 copias: tres grandes bailando en cada órbita, una sentada en los dientes de abajo y tres en las hamacas de sangre; `HW_Music_Notes`, notas que brillan alrededor de las brujas y los diablitos que bailan: copias de los modelos de [witch.blend](../../characters/halloween/witch.blend)) y `Presentation` (cámara y luces; excluir al exportar).

- **Tamaño:** factor 22 (igual que South Park); con proporciones de cráneo real mide ~42 de largo, ~31 de ancho y ~46 de alto con la mandíbula abierta. La cara mira hacia -Y y la coronilla hacia +Z.
- **Sin contorno de caricatura:** caras planas low-poly.
- **Luz `InnerGlow`** (colección `Presentation`): luz naranja de lava en el centro para ver el interior en Blender.
- **Materiales:** colores planos con prefijo `HW_`, sin texturas. La lava de afuera (`HW_Lava`, `HW_LavaHot`) y la lava de adentro (`HW_InnerLava`, `HW_InnerGlowHot` y la verde `HW_InnerGlow`) son emisivos: en el motor será el shader animado.

## Presupuesto

| Medida | Valor |
| --- | ---: |
| Triángulos (cáscara de afuera y de adentro) | ~65 000 |
| Brujas (9 copias) | ~24 300 |
| Diablitos (38 copias) | ~41 500 |
| Sangre de la boca y el mentón | ~14 000 |
| Notas musicales | ~3 400 |
| **Total de cerca** | **~148 200** |
| **Total de lejos** (`LOD_Far`: solo el cráneo) | **~22 000** |

Presupuesto de cerca: ~148 200 (los planetas de Regular Show y Star Wars tienen ~500 000). De lejos, el motor cambia a `HW_Planet_Surface_Far` (colección `LOD_Far`, oculta) y deja de dibujar personajes, notas y sangre, que a esa distancia no se ven.

## Edición

**No editar el `.blend` a mano:** se regenera completo con [build_halloween_planet.py](../../../../tools/blender/halloween/build_halloween_planet.py). Ver [instrucciones del script](../../../../tools/blender/halloween/README.md).

## Exportación para el juego

[halloween_planet.glb](exports/halloween_planet.glb): GLB para el [sistema solar](../../../../docs/solar-system.md), generado con [export_solar_models.py](../../../../tools/blender/solarsystem/export_solar_models.py) sin modificar el `.blend` (se excluyen `Presentation`, cámaras y luces). ~156 000 triángulos, más [la versión lejana](exports/halloween_planet_far.glb) (solo el cráneo de `LOD_Far`, ~22 000), que el sistema solar usa a más de 9 radios. Volver a exportar después de cambiar el modelo.
