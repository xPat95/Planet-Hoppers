> HISTÓRICO — bloqueo resuelto al conectar Blender MCP en el puerto 9876. El texto siguiente describe únicamente el primer intento; ver [entrega y verificación actuales](README.md).

# Reconstrucción del hangar: bloqueo de la etapa A

Fecha: 2026-11-02. Rama comprobada: feature/hangar.

Original localizado: assets/models/environment/spaceStation/citadel_space_station.blend.
No fue posible abrirlo en Blender durante esta sesión. No se inició el modelado ni se generaron renders; se respeta la instrucción de detenerse sin inventar un diseño alternativo. Los interiores citadel_hangar_interior.blend y citadel_hangar_phase1.blend se conservaron.

## Acceso a Blender

Blender MCP (get_addon_status y get_scene_info) respondió que no puede conectarse y requiere el complemento activo. No existe blender en PATH ni el ejecutable de Blender 5.2 indicado en scripts históricos. Se encontró el alias de Microsoft Store C:/Users/pablo/AppData/Local/Microsoft/WindowsApps/blender-launcher.exe: su ejecución restringida dio acceso denegado; con ejecución ampliada terminó sin salida, sin confirmación de bpy y sin un proceso Blender accesible. Las consultas de instalación no proporcionaron un ejecutable utilizable.

La documentación histórica del repositorio no se considera evidencia de una inspección nueva. No se afirma haber inspeccionado mallas, materiales o texturas ni haber reutilizado geometría. Ningún criterio artístico fue verificado.

## Escala comprobada mediante lectura del código

Fuentes sin modificar: src/graphics/hangar.rs, src/graphics/space.rs, src/scenes/front_end.rs.

- Plataforma: centro horizontal del juego (0, 0), radio nominal 4.5, base inferior hasta radio 4.7 y altura superior 0.8.
- Fit::Length(6.0) ajusta la mayor dimensión de cada nave a 6 unidades. Cada semieje queda como máximo en 3. Una cota conservadora de rotación horizontal es radio sqrt(18), aproximadamente 4.243; altura conservadora entre 0.8 y 6.8. Es una cota del código, no una validación de las mallas individuales.
- Cámara: FOV vertical 42 grados, dirección desde el objetivo (0.9, 0.6, 1.4) normalizada, distancia mínima 11 y margen 1.08. La distancia depende de la nave, resolución e interfaz.
- Objetivo para modelos cargados: (0, 0.8 + semialtura, 0). El placeholder tiene una corrección distinta.
- Interfaz: 151 px superiores, 156 px inferiores y una reserva horizontal de 180 px en la fórmula de encuadre.
- Conversión futura a Blender con Z vertical: (x, y, z) del juego a (x, -z, y).

## Reanudación

Se necesita Blender abierto con el complemento MCP activo, o una ruta funcional a blender.exe con ejecución de Python en segundo plano. Luego corresponde abrir el original, inspeccionar su geometría y producir vistas antes de diseñar.

La tarea queda incompleta en la etapa A. Este informe es el único archivo nuevo. No se modificaron Rust, modelos, configuraciones ni interiores anteriores. No hubo integración, commit, push, cambio de rama, merge, rebase, pull ni cargo build.
