# Estacion espacial: Ciudadela de Mass Effect

## Módulo interior de hangar

`citadel_hangar_interior.blend` y `exports/citadel_hangar_interior.glb` son un módulo complementario construido usando la fuente aprobada y sus materiales. El original `citadel_space_station.blend` permanece intacto. Rust carga el nuevo GLB en el selector y dibuja separadamente la plataforma circular histórica y la nave giratoria. [Inspección de la fuente, renders, capturas de raylib y validación](../../../../docs/references/hangar/README.md).

Modelo aprobado por el usuario el 1 de noviembre de 2026, incorporado como recurso de la estacion espacial compartida de Planet-Hoppers.

Fuente editable: [citadel_space_station.blend](citadel_space_station.blend), Blender 5.2.3 LTS. El original aprobado permanece conservado en `ProyectoFinal/Ciudadela_Mass_Effect`.

Anillo central hueco, cinco brazos con blindaje segmentado y ciudades interiores en volumen: edificios, torres, parques, canales, puentes y ventanas emisivas. La fuente mantiene las 50 mallas, materiales, cuatro camaras y luces de la version aprobada. La escena inicial de Blender tambien se conserva.

## Exportaciones

| Archivo | Uso | Triangulos |
| --- | --- | ---: |
| [GLB cercano](exports/citadel_space_station_near.glb) | Detalle completo con modificadores evaluados | 683,746 |
| [GLB lejano](exports/citadel_space_station_far.glb) | Simplificacion para distancia, misma raiz y escala | 131,699 |
| [OBJ](exports/citadel_space_station.obj) y [MTL](exports/citadel_space_station.mtl) | Alternativa de intercambio del modelo cercano | 683,746 |

El LOD lejano reduce un 80.7 % de los triangulos. Los GLB ocupan aproximadamente 39.1 MB y 10.1 MB, respectivamente. La simplificacion puede encoger los limites unos 0.05 de unidad; conserva el mismo origen y no requiere recentrar al cambiar de nivel.

Las exportaciones contienen solo la estacion: excluyen camaras, luces, fondo y objetos de la escena inicial. Los colores metalicos y las emisiones son constantes; no requieren imagenes externas. GLB conserva los materiales PBR y emisivos; OBJ/MTL tiene un soporte mas limitado de estos materiales.

## Ejes, escala y edicion

- Raiz: `CIUDADELA_ROOT | Transformacion general`. Coleccion: `CIUDADELA | Modelo editable`.
- Fuente Blender: Z superior; eje longitudinal X; brazos hacia -X.
- GLB/OBJ: Y superior; eje longitudinal X. El exportador convierte los ejes una sola vez.
- Las exportaciones se centran en la caja envolvente, con raiz en el origen y escala unitaria. La fuente conserva su pivote y coordenadas aprobados.
- Se conserva el tamaño de modelado: aproximadamente 276.75 de longitud. La escala y posicion en el sistema solar se configuraran al implementar la estacion.

La adaptacion preserva la geometria aprobada. La triangulacion, limpieza y simplificacion se aplican a copias en memoria. La fuente integrada solo actualiza metadatos de aprobacion, documentacion y la ruta relativa del render.

## Revision y alcance

[Manifest](asset.json), [auditoria de exportacion y reimportacion](../../../../docs/references/spaceStation/integration_audit.json), [vistas y referencias](../../../../docs/references/spaceStation/README.md), [scripts de construccion y exportacion](../../../../tools/blender/spaceStation/README.md).

Este paquete conserva el modelo aprobado. El sistema solar carga sus LOD como estacion fija; `config/station.json::dock` configura la compuerta inferior, aproximacion, entrada y despegue. Una bahia interior complementaria ocupa ese sector del anillo, con techo, suelo, paredes y abertura coherentes con su marco exterior, sin modificar Blender ni los GLB. La entrada desde el vuelo y el cambio de nave durante la sesion ya estan conectados. Ver `docs/station-hangar-cycle.md`. No incorpora otra tematica, planeta ni token. El LOD cercano sigue siendo de alto detalle; los recuentos y limites de ambos niveles se registran en la auditoria de exportacion, cuyo campo `runtime_placement` refleja el momento de preparacion del recurso.
