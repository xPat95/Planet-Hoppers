# Recursos derivados de SAVE KENNY!, fase 4A

Fuente editable: `southpark_threats.blend`. Contiene escenas `SouthPark_Threat_Bus`, `SouthPark_Threat_UFO`, `SouthPark_Threat_Sewer` y `SouthPark_Threat_Brake_Panel`. Seleccionar una escena en Blender y usar Material Preview para revisarla.

Exportaciones GLB, metros y Y arriba:

| Recurso | Orientación y dimensiones aproximadas | Triángulos |
| --- | --- | ---: |
| `bus.glb` | Frente +X; 5.84 m largo, 2.44 m ancho, 2.66 m alto | 3680 |
| `ufo.glb` | Centro local del platillo en origen; diámetro 5.44 m | 748 |
| `sewer.glb` | Abertura horizontal centrada en origen; diámetro exterior 1.95 m | 348 |
| `lid.glb` | Tapa horizontal centrada en origen; diámetro 1.72 m | 108 |
| `brake_panel.glb` | Poste apoyado en suelo; altura 1.83 m | 1204 |

Blender utiliza Z arriba. El generador exporta a Y arriba como el pueblo. El autobús se gira 180° en Rust para circular hacia -X. La tapa queda al lado del agujero en la escena editable y centrada en el GLB para posicionarla libremente.

Las tres bombillas del OVNI permanecen editables en Blender. El cuerpo exportado conserva sus soportes; Rust dibuja tres esferas de radio 0.25 m en las mismas posiciones y cambia su color individualmente. El rayo tractor es un cono translúcido dinámico de raylib. Estas primitivas sencillas evitan duplicar el platillo o crear texturas/modelos por cada estado.

Todos los materiales son Principled opacos de colores planos, sin imágenes ni fuentes externas. Los textos se convierten a malla. Los buffers se cargan una vez y se reutilizan durante las pruebas. `asset-audit.json` guarda los límites en coordenadas Blender y los triángulos de cada GLB; el cargador Rust exige que coincidan después de convertir los ejes.

Regenerar desde la raíz, con Blender en PATH o usando su ruta completa:

```powershell
blender --background --python-exit-code 1 --python tools/blender/southpark/build_threat_assets.py
```

Solo reemplaza los derivados de esta carpeta. No ejecutarlo sobre ediciones manuales que se quieran conservar. Reutiliza el exportador estático y su solución local para la distribución Blender Alpha disponible. Limpia selecciones en todas las escenas antes de exportar para excluir el cubo predeterminado y otros recursos.

Ver [ejecución, controles y validación](../../../../../docs/minigames/southpark-phase4a.md).
