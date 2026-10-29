# Planeta Domo Espacial

Este domo es **el planeta de Regular Show (Un Show Más) del sistema solar**. No es una nave pilotable ni el escenario implementado del minijuego; los motores forman parte del modelo visual del planeta.

Fuente editable principal: **[regularshow_space_dome_planet.blend](regularshow_space_dome_planet.blend)**, Blender 5.2.3 LTS.

## Contenido y organización

Isla rocosa de Regular Show bajo una cúpula transparente: casa principal verde, 340 árboles, senderos, estanque, fuente, snacks, caravana y taller; cuatro motores inferiores y tres propulsores horizontales traseros.

Casco rocoso; motores inferiores y traseros; parque; casa principal; vegetación; cúpula; tres claros con snacks, caravana y taller. La casa principal conserva su tamaño y las construcciones secundarias están al 65 % de su tamaño original. Los tres propulsores traseros apuntan hacia la espalda de la casa (+Y).

Todo el modelo depende de `RegularShow_Planet_ROOT`, con posición y rotación cero y escala unitaria. El pivote original se conserva en el origen; en el domo corresponde al centro del plano del parque, no al centro de su caja envolvente. Cámara, luces, fondo y cualquier escena auxiliar no forman parte del recurso exportado.

## Escala y ejes

- Diámetro nominal de la fuente: 14.4 unidades.
- Copias de juego: factor uniforme **2.2**, horneado en la geometría, con raíz unitaria. Diámetro nominal de juego: **31.68** unidades.
- Se toma como referencia el diámetro 44 de South Park y se conserva la relación de tamaños Estrella/domo de 20:14.4. Las elevaciones y accesorios pueden exceder el diámetro nominal.
- Blender: Z superior. Exportaciones OBJ/GLB: Y superior, -Z frontal. No aplicar otra conversión de ejes al reimportar mediante Blender.

## Exportaciones incluidas

| Archivo | Uso | Triángulos |
| --- | --- | ---: |
| [GLB cercano](exports/regularshow_space_dome_planet_near.glb) | Geometría limpia, UV y materiales portables | 505,230 |
| [GLB lejano](exports/regularshow_space_dome_planet_far.glb) | LOD simplificado, intercambiable por la misma raíz | 146,297 |
| [OBJ limpio](exports/regularshow_space_dome_planet.obj) y [MTL](exports/regularshow_space_dome_planet.mtl) | Alternativa de intercambio de la versión cercana | 505,230 |

El LOD lejano reduce los triángulos un **71.0 %** respecto del cercano. Los GLB contienen sus imágenes; no dependen de rutas locales. El OBJ requiere su MTL y las rutas relativas de textura que este indique. Conservar la estructura del repositorio al trasladarlo.

Colores constantes sin imágenes externas. El shader completo del vidrio permanece en la fuente Blender. GLB/MTL usan transparencia alfa simplificada: vidrio 0.09, retícula 0.5, doble cara. Se debe comprobar ordenamiento y profundidad en el motor final.

La fuente editable mantiene la geometría aprobada. La triangulación, soldadura de coincidencias microscópicas, eliminación de caras degeneradas y simplificación se realizan únicamente sobre copias en memoria para exportar. Los modelos originales de ProyectoFinal se conservaron fuera del repositorio.

## Verificación y límites

Ambos GLB fueron reimportados en Blender: se verificaron triángulos, límites y ausencia de triángulos nulos. No se han medido FPS ni validado shaders, transparencia, colisiones o cambio de LOD en un motor del juego todavía no elegido. Las versiones cercanas siguen siendo de alto detalle; el LOD debe usarse a distancia.

Ver [revisión técnica](../../../../docs/planets/regularshow.md), [diagnóstico JSON](../../../../docs/references/regularshow/regularshow_integration_audit.json) y [scripts](../../../../tools/blender/regularshow/README.md).
