# Planeta Estrella de la Muerte

Esta Estrella de la Muerte es **el planeta de Star Wars del sistema solar**. No sustituye la estación espacial compartida del hangar, no es una nave y no implementa el minijuego tipo Galaga.

Fuente editable principal: **[starwars_death_star_planet.blend](starwars_death_star_planet.blend)**, Blender 5.2.3 LTS.

## Contenido y organización

Esfera de la Estrella de la Muerte I con 7 353 placas de blindaje, trinchera ecuatorial, hangares, infraestructura y reflector cóncavo del superláser con ocho emisores.

Casco y blindaje; infraestructura y trinchera; plato del superláser. La escena original y las cámaras/luces de presentación quedan fuera de las exportaciones.

Todo el modelo depende de `StarWars_Planet_ROOT`, con posición y rotación cero y escala unitaria. El pivote original se conserva en el origen; en el domo corresponde al centro del plano del parque, no al centro de su caja envolvente. Cámara, luces, fondo y cualquier escena auxiliar no forman parte del recurso exportado.

## Escala y ejes

- Diámetro nominal de la fuente: 20 unidades.
- Copias de juego: factor uniforme **2.2**, horneado en la geometría, con raíz unitaria. Diámetro nominal de juego: **44.00** unidades.
- Se toma como referencia el diámetro 44 de South Park y se conserva la relación de tamaños Estrella/domo de 20:14.4. Las elevaciones y accesorios pueden exceder el diámetro nominal.
- Blender: Z superior. Exportaciones OBJ/GLB: Y superior, -Z frontal. No aplicar otra conversión de ejes al reimportar mediante Blender.

## Exportaciones incluidas

| Archivo | Uso | Triángulos |
| --- | --- | ---: |
| [GLB cercano](exports/starwars_death_star_planet_near.glb) | Geometría limpia, UV y materiales portables | 506,058 |
| [GLB lejano](exports/starwars_death_star_planet_far.glb) | LOD simplificado, intercambiable por la misma raíz | 69,670 |
| [OBJ limpio](exports/starwars_death_star_planet.obj) y [MTL](exports/starwars_death_star_planet.mtl) | Alternativa de intercambio de la versión cercana | 506,058 |

El LOD lejano reduce los triángulos un **86.2 %** respecto del cercano. Los GLB contienen sus imágenes; no dependen de rutas locales. El OBJ requiere su MTL y las rutas relativas de textura que este indique. Conservar la estructura del repositorio al trasladarlo.

Los dos atlas de autoría están empaquetados dentro del Blender. Las exportaciones usan doce mapas sRGB de blindaje y una normal tangente aproximada calculada a partir del microrelieve; el OBJ conserva el mapa de altura. Los otros materiales son colores constantes.

La fuente editable mantiene la geometría aprobada. La triangulación, soldadura de coincidencias microscópicas, eliminación de caras degeneradas y simplificación se realizan únicamente sobre copias en memoria para exportar. Los modelos originales de ProyectoFinal se conservaron fuera del repositorio.

## Verificación y límites

Ambos GLB fueron reimportados en Blender: se verificaron triángulos, límites y ausencia de triángulos nulos. No se han medido FPS ni validado shaders, transparencia, colisiones o cambio de LOD en un motor del juego todavía no elegido. Las versiones cercanas siguen siendo de alto detalle; el LOD debe usarse a distancia.

Ver [revisión técnica](../../../../docs/planets/starwars.md), [diagnóstico JSON](../../../../docs/references/starwars/starwars_integration_audit.json) y [scripts](../../../../tools/blender/starwars/README.md).
