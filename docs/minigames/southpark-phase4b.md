# SAVE KENNY! — fase 4B

Las cinco amenazas comparten el `ThreatDirector` existente: preparación, actividad, resolución, resultado y reinicio. Se agregaron `Piano` y `Snowman` al catálogo estable `ThreatKind::ALL`. No hay selección aleatoria, vidas, salvadas, música ni flujo definitivo de sesión.

## Ejecutar y probar

Desde la raíz, con el entorno Rust/nativo ya usado en fase 4A:

```powershell
cargo run --locked
cargo run --locked -- --threat piano
cargo run --locked -- --threat snowman
```

Los comandos directos retiran los tres objetos técnicos de fase 2. No necesitan Blender ni numpad. En laptop, usar Fn si las teclas F controlan brillo/volumen.

| Tecla | Acción |
| --- | --- |
| F5 / F6 / F7 | Autobús / OVNI / alcantarilla, conservados |
| F10 / F11 | Piano / muñeco de nieve |
| F8 | Reiniciar la amenaza seleccionada |
| F9 | Retirar el evento y restaurar paseo libre |
| WASD / mouse / Shift | Caminar / mirar / correr |
| E | Interactuar con el objetivo visible apuntado |
| Escape | Liberar/recuperar cursor; no pausa la amenaza |
| F1 / F2 / F3 / F12 | Ayuda / colisiones / indicador Kenny / captura |
| Ctrl+Q | Salir |

Solo se prepara una amenaza a la vez; cerrar primero el diálogo de Kenny. El jugador conserva sus controles FPS. Kenny se coloca en la zona de desarrollo de cada evento. Estos puntos siguen en North Avenue, suelo Y=0, Z=-42.9; no cambia la ciudad ni sus 395 colisionadores.

## Piano

Kenny empieza en (-28,0,-42.9) y recorre 16 m hacia el impacto (-12,0,-42.9). La preparación dura 3 s; el recorrido activo, 18 s. El piano permanece suspendido a 6.5 m, se balancea y muestra el área de caída. La advertencia se intensifica al superar el 75% del recorrido.

El seguro amarillo está en (-32,0,-40.9), cerca de la aparición FPS (-43.2,0,-42.9). Acercarse, apuntar al panel y pulsar **[E] Bloquear piano**. Alcance 4.5 m, oclusión y selección mediante el raycasting compartido. El panel se vuelve verde, congela el balanceo y registra `Survived`. No permite repetir la acción.

Si Kenny alcanza el centro del área sin protección, entra en resolución: se detiene allí y el piano cae con una curva cuadrática durante 0.65 s. Al tocar el suelo registra `Died`, aparece un efecto de impacto breve y Kenny se inclina/comprime antes de ocultarse. El fracaso no se registra antes de la caída. El volumen completo de caída y el acceso al seguro se validan contra la ciudad al iniciar.

Para comprobar ambos resultados manualmente: primero bloquear; F8 y después dejar que Kenny llegue sin intervenir.

## Muñeco de nieve

Kenny empieza en (-26,0,-42.9). El muñeco aparece 12 m detrás. Ambos siguen una ruta validada hasta X=22, con Kenny a 1.25 m/s y el perseguidor a 1.85 m/s. El muñeco orienta su cara hacia el movimiento y oscila ligeramente. El contacto a 1.15 m produce el fracaso y detiene la persecución; el reloj mostrado es una referencia de ventana, no una condición de muerte.

Hay tres calefactores en X=-34, -22, -10; todos en Z=-40.9. Están separados 12 m para exigir recorrer la calle. Apuntar a cada uno desde menos de 4.5 m y pulsar **[E] Encender calefactor**. El indicador se vuelve verde, el panel representa calor, el objetivo deja de seleccionarse y aumenta **Calefactores: X/3**. Una activación repetida no cuenta.

Cada calefactor reduce 0.07 m/s la velocidad del muñeco. Con los tres, se detiene, se encoge verticalmente durante 2 s y deja un charco estilizado; entonces registra `Survived`. Kenny permanece detenido durante la resolución y después continúa. Con cero, uno o dos calefactores, el muñeco aún puede alcanzarlo. La separación y velocidad dan alrededor de 18–24 s de persecución en la calle predeterminada, dependiendo de cuándo se activen los calefactores.

La ruta se calcula una sola vez por activación. Se reutilizan los nodos/enlaces de `RoadNetwork`: si el tramo directo está bloqueado, BFS busca enlaces que permitan pasar el volumen de 2.10 × 3.30 m del perseguidor. Si no hay ruta válida, se rechaza la preparación sin alterar el evento anterior. Los desplazamientos del muñeco también pasan por el barrido de colisiones existente. No hay búsqueda por frame, teletransporte del perseguidor ni física nueva. El jugador no bloquea la persecución; las superposiciones con volúmenes móviles se resuelven con el apartado lateral validado que ya usaba el autobús.

Para probar: activar los tres; F8 y dejarlo alcanzar a Kenny. F2 ayuda a revisar los volúmenes.

## Resultado, limpieza y arquitectura

F8 restaura posiciones, temporizadores, calefactores, seguro, visibilidad y animaciones. F9 elimina todos los objetivos/colisionadores `threat:*`, restaura a Kenny en un punto seguro y permite su paseo normal.

En los dos eventos nuevos, a los 6 s de mostrar el resultado se retiran automáticamente los props y se libera el control de Kenny. Si fracasó, se verifica un punto seguro próximo a su posición anterior; si el jugador lo ocupa, se intentan posiciones cercanas. Si todas están ocupadas se conserva el resultado y se reintenta, sin colocarlo dentro de obstáculos. El resultado permanece consultable; F9 también limpia su aviso. Las tres amenazas 4A conservan su comportamiento anterior de limpieza explícita.

`ThreatDirector` conserva configuración, estado, objetivos, colisionadores y control temporal de Kenny. `navigation::event_route` reutiliza el grafo. `ThreatModels` carga las cinco mallas nuevas una sola vez y reutiliza el calefactor tres veces. `StaticModel::draw_pose` permite escala y balanceo sin duplicar buffers. No se agregaron dependencias: Rust 2021, raylib-rs 5.5.1 y serde/serde_json.

Los parámetros están en `config/southpark-threats.json`; reiniciar la aplicación para aplicar cambios. Se validan valores finitos, tiempos mínimos, separación y velocidades que permitan alcanzar a Kenny incluso con dos calefactores. La geometría se mantiene en metros. Los recursos, ejes y generación reproducible están en el [README de modelos](../../assets/models/minigames/southpark/threats/phase4b/README.md).

## Verificación

```powershell
cargo fmt --all -- --check
cargo check --locked
cargo build --locked
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings
cargo run --locked -- --smoke-threats
cargo run --locked -- --smoke
cargo run --locked -- --smoke-kenny
```

Verificados: compilación, 57 pruebas (19 FPS, 15 Kenny, 23 amenazas), clippy sin advertencias y ejecución OpenGL 3.3 en Intel UHD. La prueba gráfica de amenazas ejecutó diez casos en 1697 frames, usando el raycasting compartido y eventos E de raylib; comprobó carga/dimensiones de modelos, éxito/fracaso, reinicio y retorno al paseo de los cinco tipos. La prueba FPS confirmó movimiento, carrera bloqueada, mouse, E y Escape; la de Kenny confirmó diálogo, pausa, cancelación y recuperación del cursor.

Se probaron rutas reales contra los sólidos del pueblo, caída sincronizada con Kenny, derretimiento, calefactores independientes, oclusión/distancia, limpieza de ambos resultados y movimiento a 30/60/144 FPS. También se verificó un desvío por el grafo alrededor de una pared artificial. Blender reabrió el archivo editable nuevo y reimportó los cinco GLB con error de dimensiones menor a 0.002 m. La validación existente del pueblo volvió a pasar, incluidos sus 27 materiales y 395 sólidos. Los hashes SHA-256 de los 69 recursos originales protegidos se compararon antes/después, incluidos ciudad, planeta, naves, PNG y recursos 4A.

La automatización coloca y orienta al jugador para las interacciones y usa cámaras de documentación para las capturas. **No reemplaza una partida manual con teclado/mouse físicos ni valida dificultad subjetiva o sesiones largas.** No hay daños al jugador ni simulación física del cable/nieve. La distribución de desarrollo es fija; otras zonas requieren validar sus rutas y accesos.

Capturas y reporte de ejecución:

- [Piano suspendido](../references/southpark/phase4b/piano.png), [seguro activado](../references/southpark/phase4b/piano-secured.png), [caída](../references/southpark/phase4b/piano-fall.png).
- [Persecución](../references/southpark/phase4b/snowman.png), [derretimiento](../references/southpark/phase4b/snowman-melt.png), [contacto](../references/southpark/phase4b/snowman-contact.png).
- [Diez resultados](../references/southpark/phase4b/runtime-validation.json); se conservan también las tres capturas de regresión 4A en esa carpeta. Las capturas históricas de fase 4A no se reemplazaron.
- [Verificación de FPS, Kenny, Blender y recursos originales](../references/southpark/phase4b/regression-validation.json).

## Archivos de la entrega

Modificados: `README.md`, `config/southpark-threats.json`, `src/graphics/{resources,threats}.rs`, `src/minigames/southpark/{mod,navigation,threats}.rs`, `src/scenes/{southpark_dev,threat_smoke}.rs` y `tests/southpark_threats.rs`.

Creados: este documento; `tools/blender/southpark/{build_threat_assets_4b,validate_threat_assets_4b}.py`; carpeta `assets/models/minigames/southpark/threats/phase4b/` con README, Blender, cinco GLB y auditoría; carpeta de referencias `docs/references/southpark/phase4b/` con nueve capturas y dos reportes JSON. Los informes/logs de comprobación adicionales quedan en `target/`, ignorado por Git.

Para fase 5 están disponibles los cinco identificadores estables, `ThreatKind::ALL`, resultados `Survived/Died`, consulta de estados, reinicio y limpieza. Falta implementar la sesión que consuma esos resultados, baraje sin repeticiones y gestione vidas/salvadas y presentación. No se adelantó ese flujo ni la integración solar. No se hizo commit, push ni cambio de rama.
