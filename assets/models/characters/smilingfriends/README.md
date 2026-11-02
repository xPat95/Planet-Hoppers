# Personajes de Smiling Friends

| Archivo | Qué es | Script |
| --- | --- | --- |
| `forest_demon.blend` | Forest Demon (T1 EP4), estilo plastilina, para el planeta. | `build_forest_demon.py` |
| `pim.blend` | `SF_Pim`: Pim gritando con los brazos arriba, los ojos de espiral y la boca abierta. Mira a +X. | `import_figures.py` |
| `charlie.blend` | `SF_Charlie`: Charlie saludando con la boca abierta, de frente (-Y) y con la cabeza girada a -X. | `import_figures.py` |
| `source/Pim.stl`, `source/Charlie.stl` | Las figuras originales (STL, sin colores, ~500 000 triángulos cada una), descargadas por Cami. | — |

Pim y Charlie están parados sobre z = 0, con contorno de caricatura. [`import_figures.py`](../../../../tools/blender/smilingfriends/import_figures.py) importa los STL, los rehace con caras parejas (cuadrados del mismo tamaño en toda la figura; Charlie ~151 000 triángulos, Pim ~138 000) y los pinta con bordes medidos en vistas ortogonales con grilla. Las costuras de la escultura (puños, ruedo, botamangas, capucha, gorra) se cortan con planos, así esos bordes quedan rectos. Para retocar a mano: Edit Mode, seleccionar caras (Alt+clic toma un anillo) y Assign con el color. **Si se retocan a mano, no volver a correr el script** (pisa los archivos). La nave del juego usa una copia al 40 %. La nave los importa desde estos archivos. Capturas: `docs/references/smilingfriends/pim_*.png` y `charlie_*.png` (frente, costado, atrás, tres cuartos y cara).

**No editar los `.blend` a mano:** se regeneran con sus scripts. Si se cambia un STL, hay que revisar las zonas de colores del script (están en las coordenadas del STL).
