# Nave: Caza Naboo N-1

Recurso **de nave pilotable**, separado de los planetas y de los escenarios de minijuego. Fuente editable: [naboo_n1_royal_ship.blend](naboo_n1_royal_ship.blend), Blender 5.2.3 LTS. El archivo contiene la versión aprobada, sin sobrescribir el original de ProyectoFinal.

Caza real amarillo y cromado del Episodio I, con fuselaje central alargado, cabina baja, piloto con casco granate de visor ámbar y astromecánico R2 detrás.

## Exportaciones para integración

| Archivo | Uso |
| --- | --- |
| [GLB cercano](exports/naboo_n1_royal_ship_near.glb) | Detalle completo y jerarquías de tripulantes |
| [GLB lejano](exports/naboo_n1_royal_ship_far.glb) | Copia simplificada para distancia, misma raíz y límites |
| [OBJ](exports/naboo_n1_royal_ship.obj) + [MTL](exports/naboo_n1_royal_ship.mtl) | Intercambio del detalle cercano |

Las exportaciones tienen raíz `Naboo_N1_ROOT` en el centro de su caja envolvente, escala unitaria y el mismo tamaño que la fuente. El centrado y la limpieza se aplican solo a copias en memoria. Blender usa Z superior y frente -Y; GLB/OBJ convierten a Y superior. No convertir de nuevo los ejes al reimportar mediante Blender.

No hay texturas externas. GLB conserva metal, colores, transparencia y jerarquías; OBJ/MTL requiere conservar ambos archivos juntos y ajustar metal/cristal según el visor. Se excluyen luces, cámaras, fondo y contornos de presentación del carrito.

## Personajes

El piloto de Naboo y la unidad R2 permanecen incluidos en la nave como tripulantes/accesorios. No sustituyen la asociación de Grogu con la temática Star Wars.

Los personajes separados se colocan sobre Z=0 y conservan su pose; no incluyen armature ni animaciones. Las fuentes editables permanecen dentro del Blender de la nave. El módulo compartido de personajes podrá reutilizarlos; no se duplica lógica de abducción.

## Verificación y límites

El exportador reimporta los GLB cercano, lejano y de personajes; comprueba triángulos, límites, raíces, ausencia de caras nulas, luces, cámaras y dependencias externas. [Auditoría](../../../../../docs/references/starwars/ships/naboo/integration_audit.json). [Revisión](../../../../../docs/ships/naboo_n1.md).

Todavía no hay motor elegido: no se han medido FPS ni validado transparencia, materiales, colisiones, cambio de LOD o vuelo en un ejecutable. Este paquete aporta recursos; no implementa desbloqueos, tokens ni selección de variantes.
