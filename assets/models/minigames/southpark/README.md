# SAVE KENNY! — escenario, fase 1

Recurso independiente del planeta esférico. Abrir **southpark_town.blend** en Blender.

- `southpark_town.blend`: escena editable `SouthPark_Explorable`, metros, Z arriba; iluminación y cámara de desarrollo incluidas.
- `southpark_town.glb`: únicamente geometría visual y materiales; glTF 2.0, metros, Y arriba. Sin cámara, luces, colisionadores ni dependencias de imágenes externas.
- `southpark_town.scene.json`: procedencia de cada conjunto, calles, punto inicial, límites y colisionadores simplificados en coordenadas Blender. Incluye SHA-256 del planeta fuente.

Se reutilizan los 160 edificios con sus fachadas, rótulos y detalles, 220 pinos, 14 montañas y el cartel. Los 27 materiales utilizados mantienen sus colores; el material del lago esférico no se necesita. Solo se construyen superficies planas para suelo, calles, marcas y estacionamiento.

La colección `SPT_Colliders` contiene cajas auxiliares ocultas en render y en el visor. Para inspeccionarlas, seleccionarlas desde el Outliner y desocultarlas. Las cajas son conservadoras: incluyen los salientes de cada edificio; en los pinos solo bloquea el tronco. Las montañas y el cartel también tienen cajas. Los proxies no implementan física.

Ver [guía de revisión, generación, validación y pendientes](../../../../docs/minigames/southpark-phase1.md) y [contrato de integración futura](../../../../src/minigames/southpark/README.md).
