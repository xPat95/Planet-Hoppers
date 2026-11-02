# DIAMOND RUSH — minería selectiva, fase 4 de 6

## Ejecutar y controles

```powershell
cargo run --bin minecraft -- --mine
```

WASD, mouse y Shift conservan el FPS existente. Mantener **clic izquierdo** sobre un bloque extraíble inicia los golpes; soltarlo cancela. **R** reinicia la sesión de minería, restaura todos los bloques y devuelve al spawn seguro. **Escape** libera/captura el cursor o cierra el menú. **E** frente al letrero SALIDA, o **Backspace** desde cualquier lugar, abre el menú para volver a la aldea. F12 guarda una captura.

La aldea sigue con `cargo run --bin minecraft`; su menú del desafío permanece preliminar. South Park conserva `cargo run --bin southpark` y `cargo run` mantiene el menú principal/sistema solar. La minería existe únicamente en el modo de mina; no se añade al Overworld ni a los componentes FPS generales.

## Inspección y derivado

Se confirmó `feature/minecraft-minigame` y el estado inicial limpio. Se inspeccionaron `mine.rs`, el host y HUD de la mina, los manifiestos, el cargador GLB/grupos de render, las colisiones AABB y el selector compartido de raycasting. El controlador sigue en `minigames/common/fps.rs`; no se modificaron sus controles, sensibilidad ni movimiento.

Fase 3 conservaba 14 diamantes y 12 bloques de piedra independientes, pero sus 60 minerales comunes estaban en la malla estática. Para habilitarlos sin sobrescribir recursos anteriores se creó **`assets/models/minigames/minecraft/mining/`**, con un `.blend`, GLB y manifiesto derivados.

`prepare_mining_blocks.py` abre el `.blend` de fase 3 y separa solo las 73 caras expuestas de esos 60 cubos. No redistribuye la mina ni reconstruye todos los túneles. Cada cubo recibe seis caras y un grupo `ORE_*`, compartiendo materiales/imágenes existentes. Añade fondos de piedra detrás de los nuevos nichos y recorta sus volúmenes de los colliders estructurales. Los IDs de diamantes y bloqueos de fase 3 se conservan.

Hay **86 bloques extraíbles**: 14 diamantes, 60 minerales comunes y 12 piedras de paso. Los puntos `approach` están sobre suelo transitable y evitan soportes. `occluded_by` registra bloques que obstruyen referencias de mineral; la ejecución comprueba siempre la visibilidad real mediante raycasting. Algunos minerales estaban en el revestimiento detrás de diamantes: extraer el delantero permite seleccionar y extraer el de atrás, con su fondo estructural conservado.

La copia derivada contiene 105 mallas, 20 materiales y las mismas 16 imágenes. El render mantiene **100 lotes** para conservar los grupos extraíbles. No carga imágenes ni reconstruye buffers al picar. El manifiesto tiene 391 AABB; el sistema compartido añade cuatro límites. Los PNG y los recursos de fases anteriores permanecen intactos.

## Selección y progreso

`MiningSession` contiene el estado individual de cada bloque, una extracción activa, contador y eventos. Se reutiliza `select_target`, desde la posición ocular y dirección de cámara, con alcance de **4.5 m**, limitado también por la configuración FPS. Selecciona el objeto válido más cercano y consulta las colisiones sólidas para impedir minería a través de paredes. Se excluye solo el collider del propio candidato.

El progreso se acumula por segundos transcurridos. Cambiar de objetivo, perder alcance/visibilidad, soltar el clic, abrir un menú o perder foco/captura elimina el progreso y las grietas; no se guarda trabajo parcial en otros bloques. Solo se procesa un bloque por cuadro y el tiempo sobrante no se transfiere al siguiente.

El host bloquea la minería en menús, al regresar de ellos, durante transiciones de salida y al capturar el cursor mediante clic. Después de esas acciones se requiere soltar el botón antes de volver a picar. Por eso un clic en un botón de interfaz no golpea el mineral situado detrás. Al abandonar la escena se cancela la acción y la sesión se libera.

Tiempos centralizados en **`config/minecraft-mining.json`**, modificables con `--mining-config FILE`:

| Material | Segundos |
| --- | ---: |
| Piedra marcada | 0.7 |
| Carbón | 0.9 |
| Hierro | 1.2 |
| Lapislázuli | 1.3 |
| Redstone | 1.4 |
| Oro | 1.5 |
| Diamante | 1.8 |

También se configuran alcance, objetivo provisional de ocho, período del golpe (0.32 s) y fase de impacto. Las pruebas comparan la extracción a 20, 30, 60 y 144 FPS. Los tipos y sus nombres están centralizados en `BlockKind`; no hay un mundo voxel generalizado.

## Pico, grietas y HUD

El pico de hierro es pixel art procedural de 16 × 16 celdas dibujado con cuadrados en la esquina inferior derecha. Tiene contorno oscuro, cabeza metálica iluminada y mango de madera diagonal. Se dibuja en pantalla después del mundo 3D; no choca con paredes ni tapa la mira. En reposo permanece estable. Mientras existe una extracción válida, rota repetidamente alrededor del mango; la fase de impacto coincide con el extremo del golpe. Al cancelar vuelve al reposo.

Las grietas son una red procedural de píxeles sobre las caras del bloque activo, con **ocho etapas**. La cantidad de ramificaciones crece con el progreso. Se dibujan en coordenadas mundiales con profundidad y un desplazamiento de **0.003 m** para evitar z-fighting. No se mezclan permanentemente con los materiales ni se modifican texturas. Cancelar o destruir elimina el overlay inmediatamente.

El HUD conserva la cruz central y añade nombre del material, «Mantener clic izquierdo para picar», una barra discreta y **Diamantes: X / 8**. Las grietas son el indicador principal. El contador y un mensaje breve se actualizan en el mismo cuadro de extracción. Se conserva una presentación pequeña; no se añadieron partículas o inventario.

## Destrucción, recompensa y pasadizos

Al completar la duración:

1. El bloque cambia a `alive: false` y no puede procesarse otra vez.
2. Se elimina su ID de `world.obstacles` y de los objetivos de raycasting.
3. `BlockDestroyed` hace que el host oculte su grupo mediante `set_group_visible`, conservando los buffers estáticos.
4. Se cancela la referencia activa y el siguiente rayo puede alcanzar minerales situados detrás.
5. Diamond ore suma exactamente un diamante; los otros materiales no incrementan ese contador.

Suelo, techos, soportes esenciales, paredes normales y límites no forman parte de la lista extraíble. Los huecos de mineral tienen fondo y los colliders estructurales no cubren los volúmenes extraíbles. Los dos tabiques de piedra conservan sus bloques individuales; retirar una columna con suficiente altura permite pasar físicamente. `PassageUnlocked` se emite una vez por pasadizo cuando el volumen real del jugador cabe en una columna libre. Limpiar ambos tabiques completos permite un acceso más ancho.

El contador **no se limita a ocho** y no activa una victoria. La prueba gráfica llega a nueve y sigue explorando; una prueba automatizada extrae los 86 bloques permitidos y alcanza 14 diamantes sin eliminar estructura.

## Reinicio y eventos futuros de audio

`reset_mining_session()` restaura las copias iniciales de colliders y objetivos, reactiva los 86 bloques y emite `SessionReset` para mostrar sus grupos. El contador vuelve a cero; desaparecen progreso, grietas, mensajes y golpes. El jugador vuelve al spawn para evitar quedar dentro de un tabique restaurado. No se recargan texturas ni se requiere cerrar la aplicación. R activa esta función únicamente con controles de exploración habilitados.

Los eventos tipados que consume el host quedan disponibles para conectar audio en fase 5:

- `SwingStarted`: inicio de cada golpe.
- `Impact`: momento de contacto durante el golpe.
- `BlockDestroyed`: ID y material retirado.
- `DiamondCollected`: diamante y contador actualizado.
- `CommonOreCollected`: material común extraído.
- `PassageUnlocked`: apertura física de un paso.
- `SessionReset`: limpieza/restauración de sesión.

Se entregan mediante `drain_events()` por cuadro. No se implementó un bus general ni se agregaron/reprodujeron sonidos provisionales. No hay nuevos MP3/WAV.

## Validación y reproducción

```powershell
cargo check --offline --locked
cargo build --offline --locked
cargo test --offline --locked
cargo run --bin minecraft -- --mine --smoke-mining
```

La prueba de OpenGL usa el mismo bucle de entrada, HUD, render y eventos de raylib en una ventana oculta. Comprueba clic sostenido, liberación, grietas, desaparición del grupo, retirada del collider/objetivo, nueve diamantes, los cinco minerales comunes, dos pasos abiertos, recorrido por un paso, cancelación por menú, protección contra clics de UI, R y retorno a la aldea. Se usan poses de prueba para pasar entre casos; no es una partida manual autónoma. Las capturas están en [el índice de fase 4](../references/minecraft/phase4/README.md).

Las pruebas de `tests/minecraft_mining.rs` comprueban tiempos, reinicio al soltar/cambiar objetivo, alcance, paredes, obstrucción sobrevenida, pérdida de captura, eliminación de collider/target, recompensas únicas, minerales comunes, apertura de pasos, restauración, independencia de FPS y eventos. La prueba de todos los bloques sigue sus dependencias y usa raycasting real desde posiciones transitables; la prueba de un mineral oculto confirma que el diamante delantero debe retirarse primero.

`validate_mining_blocks.py` reabre el derivado, comprueba los 86 cubos y sus colliders exclusivos, los fondos de los 74 nichos al ocultar minerales en una escena descartable, UV y Nearest, y reimporta el GLB para comparar objetos, materiales, triángulos, orientación y escala. Los 227 archivos protegidos se comparan contra sus SHA-256 originales.

Se ejecutó `cargo fmt --check`: detecta diferencias preexistentes en archivos ajenos, como `src/audio.rs`. No se reformatearon esos archivos. La comprobación equivalente **`rustfmt --check --config skip_children=true`** pasa sobre todos los archivos Rust nuevos/modificados de esta fase. Los comandos, códigos de salida y resultados de la suite completa se registran en `verification.json`; los modos gráficos de regresión se resumen en `regression.json`.

Para regenerar o revisar los recursos con Blender:

```powershell
blender --background --factory-startup --python-exit-code 1 --python tools/blender/minecraft/prepare_mining_blocks.py
blender --background --factory-startup --python-exit-code 1 --python tools/blender/minecraft/validate_mining_blocks.py
```

El generador solo reemplaza este derivado. Conservar una copia si se hicieron ediciones manuales que se quieran guardar. No ejecutar el generador de fase 3 para preparar fase 4.

## Archivos y límites

Nuevos: `src/minigames/minecraft/mining.rs`, `src/graphics/mining.rs`, `src/scenes/minecraft_mining_smoke.rs`, `config/minecraft-mining.json`, `tests/minecraft_mining.rs`, los dos scripts Blender, recursos bajo `assets/models/minigames/minecraft/mining/`, esta guía y capturas/informes bajo `docs/references/minecraft/phase4/`.

Modificados: `src/minigames/minecraft/mine.rs` admite los tipos comunes y referencias de obstrucción; `minecraft/mod.rs` y `graphics/mod.rs` registran los módulos; `graphics/resources.rs` expone la consulta de visibilidad para validación; `minecraft_mine_dev.rs` compone minería/feedback/reset y usa el derivado; `minecraft_mine_smoke.rs` mantiene sus pruebas compatibles con más bloques individuales. No se cambian las colisiones o el raycasting generales, Cargo.toml/Cargo.lock, South Park o el sistema solar.

No hubo una sesión manual prolongada con hardware físico. El pico es pixel art en pantalla, las grietas son procedurales, la lava permanece estática e inofensiva y el suelo horizontal conserva las limitaciones de fase 3. Las referencias de aproximación y los fondos se validaron también para los minerales que quedaban detrás de otros bloques.

Para fase 5 quedan balanceo, temporizador, regreso como condición de victoria, peligros y conexión de audio según se autorice. En esta entrega no hay daño, salud, victoria/derrota, inventario, música, efectos sonoros ni integración definitiva con el sistema solar. Se trabajó en `feature/minecraft-minigame`, sin commit ni push.
