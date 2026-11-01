# Contrato del escenario de South Park

La fase 4A incorpora `threats.rs`: un coordinador para autobús, OVNI y alcantarilla, resultados individuales y reinicio. Usa los mismos objetivos/raycasting y controla temporalmente al actor de fase 3. Ver [pruebas y controles](../../../docs/minigames/southpark-phase4a.md).

La fase 3 añade `kenny.rs` (estado, movimiento y animación), `navigation.rs` (waypoints de las calles) y el diálogo de SAVE KENNY!. El billboard reutilizable está en `src/graphics/billboard.rs`; el raycasting común permite refinar el impacto con el alfa del PNG. No hay amenazas ni vidas. Ver [guía de fase 3](../../../docs/minigames/southpark-phase3.md).

La fase 1 entregó el contrato de recursos descrito abajo. La fase 2 lo consume desde Rust/raylib-rs: `mod.rs` compone el escenario y los objetos técnicos; `src/scenes/southpark_dev.rs` ofrece la ejecución independiente. El controlador FPS, las colisiones y la selección por rayo viven en `src/minigames/common/`. Todavía no existe un administrador global de escenas ni integración con el sistema solar. Ver [ejecución, configuración y verificación](../../../docs/minigames/southpark-phase2.md).

El punto de entrada de recursos es `assets/models/minigames/southpark/southpark_town.scene.json`, identificador `southpark.save-kenny.environment`, esquema 1. Las rutas `blend` y `visual_asset` se resuelven respecto de ese manifiesto; `source` es una ruta de auditoría relativa a la raíz del repositorio. No se necesita cargar el planeta fuente durante el juego.

Cuando se elija el motor, el adaptador de `src/scenes/` deberá:

1. Recibir y conservar el contexto de entrada del sistema solar: nave activa, posición y orientación de regreso, y referencias al estado compartido. El escenario no modifica ese contexto.
2. Cargar `visual_asset` bajo un contenedor propio de escena y crear su iluminación. La iluminación de Blender es de desarrollo y no viaja dentro del GLB.
3. Convertir los datos del manifiesto de Z arriba a Y arriba si el motor utiliza los ejes glTF: `(x,y,z) → (x,z,-y)`. Para una AABB, transformar sus ocho vértices y volver a calcular mínimos y máximos; no transformar solo dos esquinas. Los tamaños se expresan en metros.
4. Usar `spawn.position`, `spawn.forward` y `spawn.eye_height` para el controlador futuro. No se entrega un controlador en esta fase.
5. Registrar una superficie horizontal en Z=0 dentro de los límites del collider `walkable`. Calles, marcas y estacionamiento son capas visuales casi coplanares; no requieren colisionadores apilados.
6. Convertir las entradas `role: solid`, `shape: aabb` en obstáculos estáticos. Las cajas proceden de edificios completos, troncos, cartel y montañas. La decoración restante y las copas de árboles no bloquean al jugador. Las cajas son una primera aproximación; los porches podrán descomponerse en fase 2 si es necesario.
7. Facilitar una salida mediante el evento conceptual `requestReturn`, con `planet: southpark`, `result: null` y el contexto de entrada original. Este es un contrato reservado, no una integración ni un evento de victoria implementado. La escena anfitriona decide cómo regresar.
8. Al salir, liberar el contenedor, recursos gráficos, colisionadores y entradas que haya creado el adaptador. Los materiales compartidos entre instancias deberán liberarse una sola vez.

El manifiesto también conserva `roads`, `assets` y `relocations` para revisar el trazado y relacionar cada recurso derivado con el original. No contiene personajes, amenazas, HUD ni lógica del minijuego. Mantener las reglas futuras de SAVE KENNY! en esta carpeta; movimiento y colisiones reutilizables corresponderán a `src/minigames/common/`, sin duplicar el progreso de `src/gameplay/`.
