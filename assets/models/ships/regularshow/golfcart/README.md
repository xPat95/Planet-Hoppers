# Nave: Carrito de Regular Show

Recurso **de nave pilotable**, separado de los planetas y de los escenarios de minijuego. Fuente editable: [regularshow_golfcart_ship.blend](regularshow_golfcart_ship.blend), Blender 5.2.3 LTS. El archivo contiene la versión aprobada, sin sobrescribir el original de ProyectoFinal.

Carrito de golf marfil con alas de cartón corrugado sujetas con cinta, cubeta de mantenimiento y herramientas. Mordecai conduce y Rigby va sobre el techo.

## Exportaciones para integración

| Archivo | Uso |
| --- | --- |
| [GLB cercano](exports/regularshow_golfcart_ship_near.glb) | Detalle completo y jerarquías de tripulantes |
| [GLB lejano](exports/regularshow_golfcart_ship_far.glb) | Copia simplificada para distancia, misma raíz y límites |
| [OBJ](exports/regularshow_golfcart_ship.obj) + [MTL](exports/regularshow_golfcart_ship.mtl) | Intercambio del detalle cercano |

Las exportaciones tienen raíz `RegularShow_GolfCart_ROOT` en el centro de su caja envolvente, escala unitaria y el mismo tamaño que la fuente. El centrado y la limpieza se aplican solo a copias en memoria. Blender usa Z superior y frente -Y; GLB/OBJ convierten a Y superior. No convertir de nuevo los ejes al reimportar mediante Blender.

No hay texturas externas. GLB conserva metal, colores, transparencia y jerarquías; OBJ/MTL requiere conservar ambos archivos juntos y ajustar metal/cristal según el visor. Se excluyen luces, cámaras, fondo y contornos de presentación del carrito.

## Personajes

- [Mordecai](../../../characters/regularshow/mordecai.glb), copia independiente en la pose modelada.
- [Rigby](../../../characters/regularshow/rigby.glb), copia independiente en la pose modelada.

Los personajes separados se colocan sobre Z=0 y conservan su pose; no incluyen armature ni animaciones. Las fuentes editables permanecen dentro del Blender de la nave. El módulo compartido de personajes podrá reutilizarlos; no se duplica lógica de abducción.

## Verificación y límites

El exportador reimporta los GLB cercano, lejano y de personajes; comprueba triángulos, límites, raíces, ausencia de caras nulas, luces, cámaras y dependencias externas. [Auditoría](../../../../../docs/references/regularshow/ships/golfcart/integration_audit.json). [Revisión](../../../../../docs/ships/regularshow_golfcart.md).

Todavía no hay motor elegido: no se han medido FPS ni validado transparencia, materiales, colisiones, cambio de LOD o vuelo en un ejecutable. Este paquete aporta recursos; no implementa desbloqueos, tokens ni selección de variantes.
