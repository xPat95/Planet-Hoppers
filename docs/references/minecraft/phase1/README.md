# Revisión visual — DIAMOND RUSH, fase 1

Abrir los PNG directamente en cualquier visor, o seguir estos enlaces:

- [Vista general: cinco edificios](overview.png).
- [Nivel del jugador](player.png).
- [Entrada de mina](mine.png).
- [Zona agrícola](farm.png).
- [Centro y pozo](center.png).

Son renders de Blender con geometría y materiales reales. No son capturas de una escena FPS ejecutándose en Rust.

[validation.json](validation.json) registra las comprobaciones automáticas. [texture_inventory.json](texture_inventory.json) enumera las dimensiones reales de las 38 imágenes disponibles. [protected_sources.json](protected_sources.json) conserva los hashes previos de los 170 archivos protegidos: modelos originales de Minecraft/South Park, exportaciones, fuentes Rust, herramientas de South Park y texturas Minecraft. El validador exige que permanezcan iguales.

La [guía de fase 1](../../../minigames/minecraft-phase1.md) explica escala, contrato, regeneración y limitaciones.
