# Nave de Butters — Planet Hoppers

Fuente editable: [southpark_butters_ship.blend](southpark_butters_ship.blend), creada con Blender 5.4.0 Alpha. Referencia principal: imagen concepto de la nave Butters adjunta por el usuario en la conversación.

Caja rectangular cerrada, cara frontal de Butters y mechón amarillo de papel doblado sobre el capó. Cabina con techo, panel trasero y tres ventanas de plástico ahumado con transmisión. Dos latas rojas recicladas como propulsores, collares grises, boquillas abiertas con fondo oscuro, dos estabilizadores pequeños y aleta trasera. Cinta marfil, bordes corrugados, emblemas SP y doodles sencillos.

## Materiales y textura

15 materiales del modelo: cartón kraft, bordes oscuros, papel de los cantos, cinta marfil, tinta, morado, amarillo del pelo, crayón amarillo, ojos blancos, latas rojas, collares reciclados, hollín, plástico transparente ahumado, reflejos y asiento verde azulado. El cartón y la cinta no son metálicos; solo los collares tienen una respuesta metálica tenue.

Una textura propia de cartón de 256 × 256 px, [butters_kraft_paper.png](../../../../textures/ships/southpark/butters/butters_kraft_paper.png). Está empaquetada dentro del Blender e incorporada al GLB. El resto son colores de materiales y geometría, sin fuentes tipográficas externas ni modelos descargados. Los dibujos y las letras ya son mallas.

## Organización y uso

La escena `SouthPark_Butters_Ship` contiene la colección `NAVE SOUTH PARK | Butters`, con subcolecciones Hull, Cabin, Face, Tape, Engines, Fins y Doodles. Los objetos del vehículo dependen de `SouthPark_Butters_ROOT`, pivote interior cercano al centro del vehículo, con escala unitaria. Frente −Y, arriba +Z; unidades en metros. Tamaño aproximado: 4.18 m de ancho incluyendo alas, 5.07 m de largo y 3.38 m de alto. La colección PRESENTATION reúne suelo, luces y cámara; queda excluida del GLB.

[GLB para integración posterior](exports/southpark_butters_ship.glb): 15 mallas agrupadas por material, 10 374 triángulos y textura incorporada. El Blender conserva 206 mallas separadas para edición. No se añade lógica, física, colisiones ni animación. El exportador convierte a Y superior de glTF; no volver a convertir ejes al importarlo en Blender.

La [auditoría](../../../../../docs/references/southpark/ships/butters/integration_audit.json) comprueba la reimportación, límites, raíz, ausencia de triángulos degenerados, textura incorporada y exclusión de luces y cámaras. Comprueba también que la exportación deja el archivo fuente intacto. Falta validar la transmisión del plástico, las colisiones y el rendimiento en el motor final.

## Capturas y archivos agregados

[Carpeta de capturas](../../../../../docs/references/southpark/ships/butters/README.md): vistas 3/4 frontal, lateral, frontal y 3/4 trasera, a 1280 × 960.

Todos los archivos creados pertenecen a esta nave:

- `assets/models/ships/southpark/butters/`: Blender, GLB y esta nota.
- `assets/textures/ships/southpark/butters/`: textura de cartón.
- `docs/references/southpark/ships/butters/`: cuatro PNG, auditoría JSON y nota de vistas.
- `tools/blender/southpark/ships/butters/`: generador, capturador, exportador y sus instrucciones.

No se modificaron archivos existentes de otras naves ni planetas. No se realizó commit ni push.
